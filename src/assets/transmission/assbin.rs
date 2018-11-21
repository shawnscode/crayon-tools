use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crayon::errors::*;
use crayon::math::prelude::*;
use crayon::video::assets::mesh::*;
use crayon::video::assets::shader::Attribute;

use crayon_world::assets::prefab::PrefabNode;
use crayon_world::prelude::Transform;

const ASSBIN_CHUNK_AISCENE: u32 = 0x1239;
const ASSBIN_CHUNK_AINODE: u32 = 0x123c;
const ASSBIN_CHUNK_AIMESH: u32 = 0x1237;
// const ASSBIN_CHUNK_AITEXTURE: u32 = 0x1236;

const ASSBIN_MESH_HAS_POSITIONS: u32 = 0x1;
const ASSBIN_MESH_HAS_NORMALS: u32 = 0x2;
const ASSBIN_MESH_HAS_TANGENTS_AND_BITANGENTS: u32 = 0x4;

const ASSBIN_MESH_HAS_TEXCOORDS: [u32; 8] =
    [0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000, 0x4000, 0x8000];
const ASSBIN_MESH_HAS_COLORS: [u32; 8] = [
    0x10000, 0x20000, 0x40000, 0x80000, 0x100000, 0x200000, 0x400000, 0x800000,
];

impl_vertex!{
    AssbinVertex {
        position => [Position; Float; 3; false],
        normal => [Normal; Float; 3; false],
        tangent => [Tangent; Float; 3; false],
        bitangent => [Bitangent; Float; 3; false],
        color0 => [Color0; Float; 4; false],
        color1 => [Color1; Float; 4; false],
        texcoord0 => [Texcoord0; Float; 2; false],
        texcoord1 => [Texcoord1; Float; 2; false],
        texcoord2 => [Texcoord2; Float; 2; false],
        texcoord3 => [Texcoord3; Float; 2; false],
    }
}

pub struct AssbinMetadata {
    pub meshes: Vec<String>,
}

impl AssbinMetadata {
    pub fn load<R: Read + Seek>(mut file: &mut R) -> Result<AssbinMetadata> {
        file.seek(SeekFrom::Current(512))?;

        // Magic chunk ID (ASSBIN_CHUNK_XXX)
        let chunk_id = file.read_u32::<LittleEndian>()?;
        assert!(chunk_id == ASSBIN_CHUNK_AISCENE);
        // Chunk data length, in bytes
        file.read_u32::<LittleEndian>()?;

        // Flags
        let _flags = file.read_u32::<LittleEndian>()?;
        let num_meshes = file.read_u32::<LittleEndian>()?;
        let _num_materials = file.read_u32::<LittleEndian>()?;
        let _num_animations = file.read_u32::<LittleEndian>()?;
        let _num_textures = file.read_u32::<LittleEndian>()?;
        let _num_lits = file.read_u32::<LittleEndian>()?;
        let _num_cameras = file.read_u32::<LittleEndian>()?;

        //
        let mut nodes = Vec::new();
        Assbin::load_node(&mut file, &mut nodes)?;

        let mut meshes = Vec::new();
        for i in 0..num_meshes {
            meshes.push(Assbin::find_mesh_name(&nodes, i as usize));
        }

        Ok(AssbinMetadata { meshes: meshes })
    }
}

#[derive(Debug, Clone)]
pub struct Assbin {
    pub version_major: u32,
    pub version_minor: u32,
    pub version_reversion: u32,

    pub nodes: Vec<PrefabNode>,
    pub meshes: Vec<(String, MeshParams, MeshData)>,
}

impl Assbin {
    pub fn load<R: Read + Seek>(mut file: &mut R) -> Result<Assbin> {
        // SIGNATURE
        file.seek(SeekFrom::Current(44))?;

        let mut assbin = Assbin {
            version_major: file.read_u32::<LittleEndian>()?,
            version_minor: file.read_u32::<LittleEndian>()?,
            version_reversion: file.read_u32::<LittleEndian>()?,
            nodes: Vec::new(),
            meshes: Vec::new(),
        };

        // Assimp compile flags.
        file.read_u32::<LittleEndian>()?;

        let shortened = file.read_u16::<LittleEndian>()?;
        assert!(shortened == 0, "Shortened binaries are not supported!");

        let compressed = file.read_u16::<LittleEndian>()?;
        assert!(compressed == 0, "Compressed binaries are not supported!");

        // Zero-terminated source file name, UTF-8
        file.seek(SeekFrom::Current(256))?;
        // Zero-terminated command line parameters passed to assimp_cmd, UTF-8
        file.seek(SeekFrom::Current(128))?;
        // Paddings
        file.seek(SeekFrom::Current(64))?;

        assbin.load_scene(&mut file)?;
        Ok(assbin)
    }
}

impl Assbin {
    fn load_scene<R: Read + Seek>(&mut self, mut file: &mut R) -> Result<()> {
        // Magic chunk ID (ASSBIN_CHUNK_XXX)
        let chunk_id = file.read_u32::<LittleEndian>()?;
        assert!(chunk_id == ASSBIN_CHUNK_AISCENE);
        // Chunk data length, in bytes
        file.read_u32::<LittleEndian>()?;

        // Flags
        let _flags = file.read_u32::<LittleEndian>()?;
        let num_meshes = file.read_u32::<LittleEndian>()?;
        let _num_materials = file.read_u32::<LittleEndian>()?;
        let _num_animations = file.read_u32::<LittleEndian>()?;
        let _num_textures = file.read_u32::<LittleEndian>()?;
        let _num_lits = file.read_u32::<LittleEndian>()?;
        let _num_cameras = file.read_u32::<LittleEndian>()?;

        Assbin::load_node(&mut file, &mut self.nodes)?;

        for i in 0..num_meshes {
            let (params, data) = Assbin::load_mesh(&mut file)?;
            let name = Assbin::find_mesh_name(&self.nodes, i as usize);
            self.meshes.push((name, params, data));
        }

        Ok(())
    }

    fn find_mesh_name(nodes: &[PrefabNode], index: usize) -> String {
        for v in nodes {
            if v.mesh_renderer == Some(index) {
                return v.name.clone();
            }
        }

        format!("mesh_{}", index)
    }

    fn load_node<R: Read + Seek>(file: &mut R, nodes: &mut Vec<PrefabNode>) -> Result<usize> {
        // Magic chunk ID (ASSBIN_CHUNK_XXX)
        let chunk_id = file.read_u32::<LittleEndian>()?;
        assert!(chunk_id == ASSBIN_CHUNK_AINODE);

        // Chunk data length, in bytes
        let chunk_size = file.read_u32::<LittleEndian>()?;
        let chunk_cursor = file.seek(SeekFrom::Current(0))?;

        let mut name = Assbin::read_str(file)?;
        if let Some(index) = name.find("_$AssimpFbx$_") {
            name.truncate(index);
        }

        let local_transform = Assbin::read_transform(file)?;
        let num_children = file.read_u32::<LittleEndian>()?;
        let num_meshes = file.read_u32::<LittleEndian>()?;
        file.read_u32::<LittleEndian>()?;

        let mut node = PrefabNode {
            name: name,
            local_transform: local_transform,
            first_child: None,
            next_sib: None,
            mesh_renderer: None,
        };

        for _ in 0..num_meshes {
            let mesh_index = file.read_u32::<LittleEndian>()?;
            if node.mesh_renderer.is_none() {
                node.mesh_renderer = Some(mesh_index as usize);
            }
        }

        let idx = nodes.len();
        nodes.push(node);

        let mut first_child_idx = None;
        let mut child_idx = None;
        for _ in 0..num_children {
            let i = Assbin::load_node(file, nodes)?;

            if first_child_idx.is_none() {
                first_child_idx = Some(i);
            }

            if let Some(prev_idx) = child_idx {
                let node: &mut PrefabNode = nodes.get_mut(prev_idx).unwrap();
                node.next_sib = Some(i);
            }

            child_idx = Some(i);
        }

        nodes[idx].first_child = first_child_idx;
        file.seek(SeekFrom::Start(chunk_cursor + chunk_size as u64))?;
        Ok(idx)
    }

    fn load_mesh<R: Read + Seek>(mut file: &mut R) -> Result<(MeshParams, MeshData)> {
        // Magic chunk ID (ASSBIN_CHUNK_XXX)
        let chunk_id = file.read_u32::<LittleEndian>()?;
        assert!(chunk_id == ASSBIN_CHUNK_AIMESH);

        // Chunk data length, in bytes
        let chunk_size = file.read_u32::<LittleEndian>()?;
        let chunk_cursor = file.seek(SeekFrom::Current(0))?;

        let primitive = file.read_u32::<LittleEndian>()?;
        let num_vertices = file.read_u32::<LittleEndian>()? as usize;
        let num_faces = file.read_u32::<LittleEndian>()?;
        let _num_bones = file.read_u32::<LittleEndian>()?;
        let _mat_index = file.read_u32::<LittleEndian>()?;

        let mut buf = Vec::with_capacity(num_vertices);
        buf.resize(num_vertices, AssbinVertex::default());

        let mut layout = VertexLayout::build();
        let mut aabb = Aabb3::zero();

        let attributes = file.read_u32::<LittleEndian>()?;
        if (attributes & ASSBIN_MESH_HAS_POSITIONS) != 0 {
            layout = layout.with(Attribute::Position, VertexFormat::Float, 3, false);

            for i in 0..num_vertices {
                buf[i].position = Assbin::read_vec3(&mut file)?;
                aabb = aabb.grow(buf[i].position.into());
            }
        }

        if (attributes & ASSBIN_MESH_HAS_NORMALS) != 0 {
            layout = layout.with(Attribute::Normal, VertexFormat::Float, 3, false);

            for i in 0..num_vertices {
                buf[i].normal = Assbin::read_vec3(&mut file)?;
            }
        }

        if (attributes & ASSBIN_MESH_HAS_TANGENTS_AND_BITANGENTS) != 0 {
            layout = layout.with(Attribute::Tangent, VertexFormat::Float, 3, false);
            layout = layout.with(Attribute::Bitangent, VertexFormat::Float, 3, false);

            for i in 0..num_vertices {
                buf[i].tangent = Assbin::read_vec3(&mut file)?;
            }

            for i in 0..num_vertices {
                buf[i].bitangent = Assbin::read_vec3(&mut file)?;
            }
        }

        for i in 0..ASSBIN_MESH_HAS_COLORS.len() {
            if (attributes & ASSBIN_MESH_HAS_COLORS[i]) == 0 {
                break;
            }

            match i {
                0 => layout = layout.with(Attribute::Color0, VertexFormat::Float, 4, false),
                1 => layout = layout.with(Attribute::Color1, VertexFormat::Float, 4, false),
                _ => {}
            }

            for j in 0..num_vertices {
                let v = Assbin::read_vec4(&mut file)?;
                match i {
                    0 => buf[j].color0 = v,
                    1 => buf[j].color1 = v,
                    _ => {}
                }
            }
        }

        for i in 0..ASSBIN_MESH_HAS_TEXCOORDS.len() {
            if (attributes & ASSBIN_MESH_HAS_TEXCOORDS[i]) == 0 {
                break;
            }

            match i {
                0 => layout = layout.with(Attribute::Texcoord0, VertexFormat::Float, 2, false),
                1 => layout = layout.with(Attribute::Texcoord1, VertexFormat::Float, 2, false),
                2 => layout = layout.with(Attribute::Texcoord2, VertexFormat::Float, 2, false),
                3 => layout = layout.with(Attribute::Texcoord3, VertexFormat::Float, 2, false),
                _ => {}
            }

            for j in 0..num_vertices {
                let v = Assbin::read_vec3(&mut file)?;
                match i {
                    0 => buf[j].texcoord0 = [v[1], v[2]],
                    1 => buf[j].texcoord1 = [v[1], v[2]],
                    2 => buf[j].texcoord2 = [v[1], v[2]],
                    3 => buf[j].texcoord3 = [v[1], v[2]],
                    _ => {}
                }
            }
        }

        let mut indices = Vec::new();
        for _ in 0..num_faces {
            let num_indices = file.read_u16::<LittleEndian>()? as usize;
            indices.reserve(num_indices);
            for _ in 0..num_indices {
                indices.push(file.read_u16::<LittleEndian>()?);
            }
        }

        file.seek(SeekFrom::Start(chunk_cursor + chunk_size as u64))?;

        let mut vertices = Vec::new();
        for v in &buf {
            if (attributes & ASSBIN_MESH_HAS_POSITIONS) != 0 {
                Assbin::write_f32_slice(&mut vertices, &v.position)?;
            }

            if (attributes & ASSBIN_MESH_HAS_NORMALS) != 0 {
                Assbin::write_f32_slice(&mut vertices, &v.normal)?;
            }

            if (attributes & ASSBIN_MESH_HAS_TANGENTS_AND_BITANGENTS) != 0 {
                Assbin::write_f32_slice(&mut vertices, &v.tangent)?;
                Assbin::write_f32_slice(&mut vertices, &v.bitangent)?;
            }

            if (attributes & ASSBIN_MESH_HAS_COLORS[0]) != 0 {
                Assbin::write_f32_slice(&mut vertices, &v.color0)?;
            }

            if (attributes & ASSBIN_MESH_HAS_COLORS[1]) != 0 {
                Assbin::write_f32_slice(&mut vertices, &v.color1)?;
            }

            if (attributes & ASSBIN_MESH_HAS_TEXCOORDS[0]) != 0 {
                Assbin::write_f32_slice(&mut vertices, &v.texcoord0)?;
            }

            if (attributes & ASSBIN_MESH_HAS_TEXCOORDS[1]) != 0 {
                Assbin::write_f32_slice(&mut vertices, &v.texcoord1)?;
            }

            if (attributes & ASSBIN_MESH_HAS_TEXCOORDS[2]) != 0 {
                Assbin::write_f32_slice(&mut vertices, &v.texcoord2)?;
            }

            if (attributes & ASSBIN_MESH_HAS_TEXCOORDS[3]) != 0 {
                Assbin::write_f32_slice(&mut vertices, &v.texcoord3)?;
            }
        }

        let mut params = MeshParams::default();
        params.num_verts = num_vertices;
        params.num_idxes = indices.len();
        params.layout = layout.finish();
        params.aabb = aabb;
        params.primitive = match primitive {
            1 => MeshPrimitive::Points,
            2 => MeshPrimitive::Lines,
            4 => MeshPrimitive::Triangles,
            _ => unreachable!(),
        };

        let data = MeshData {
            vptr: vertices.into_boxed_slice(),
            iptr: IndexFormat::encode(&indices).into(),
        };

        params.validate(Some(&data))?;
        Ok((params, data))
    }

    fn read_vec3<R: Read + Seek>(file: &mut R) -> Result<[f32; 3]> {
        Ok([
            file.read_f32::<LittleEndian>()?,
            file.read_f32::<LittleEndian>()?,
            file.read_f32::<LittleEndian>()?,
        ])
    }

    fn read_vec4<R: Read + Seek>(file: &mut R) -> Result<[f32; 4]> {
        Ok([
            file.read_f32::<LittleEndian>()?,
            file.read_f32::<LittleEndian>()?,
            file.read_f32::<LittleEndian>()?,
            file.read_f32::<LittleEndian>()?,
        ])
    }

    fn read_str<R: Read + Seek>(file: &mut R) -> Result<String> {
        let n = file.read_u32::<LittleEndian>()?;
        let mut bytes = Vec::new();
        for _ in 0..n {
            bytes.push(file.read_u8()?);
        }

        Ok(String::from_utf8(bytes)?)
    }

    fn read_transform<R: Read + Seek>(file: &mut R) -> Result<Transform> {
        for _ in 0..4 {
            for _ in 0..4 {
                file.read_f32::<LittleEndian>()?;
            }
        }

        let transform = Transform::default();
        Ok(transform)
    }

    fn write_f32_slice<W: Write>(file: &mut W, slice: &[f32]) -> Result<()> {
        for v in slice {
            file.write_f32::<LittleEndian>(*v)?;
        }

        Ok(())
    }
}
