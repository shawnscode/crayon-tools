use assets::AssetParams;

/// Settings of transmission importing.
#[derive(Default, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TransmissionImportParams {
    pub mesh: MeshImportParams,
}

/// Settings of importing mesh resources.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MeshImportParams {
    /// The vertices and indices will be reordered for better GPU performance.
    /// Techniques that require strict vertex ordering like mesh morphing or
    /// special particle mesh emitter effects should have this option disabled.
    pub optimize: bool,
    /// Genreates normals.
    pub calculate_normals: bool,
    /// Generates tangents.
    pub calculate_tangents: bool,
    /// Generates uv0.
    pub calculate_texcoord: bool,
}

impl Default for MeshImportParams {
    fn default() -> Self {
        MeshImportParams {
            optimize: true,
            calculate_normals: false,
            calculate_tangents: false,
            calculate_texcoord: false,
        }
    }
}

impl From<AssetParams> for TransmissionImportParams {
    fn from(params: AssetParams) -> Self {
        match params {
            AssetParams::Transmission(params) => params,
            _ => TransmissionImportParams::default(),
        }
    }
}
