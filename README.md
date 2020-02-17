# What's This?

Various command line tools for [crayon](https://github.com/shawnscode/crayon).

### Getting started

Its easy to setup `crayon-cli` in your environments by following steps below:

``` sh
git clone git@github.com:shawnscode/crayon-tools.git && make
```
For windows, you will need use administration console.

### Workspace

We are using a simple `workspace.toml` file to configurate the workspace settings. Here is a minimal version of it:

```toml
[assets]
source = 'assets' # the path to assets folder..
destination = 'resources' # the path to resources folder, which are usually compiled from assets.

# sets the importer for extensions, if extensions are not listed below, it will be treated as
# `Bytes` asset.
[[assets.importers]]
type = 'Texture'
extensions = ['.png', '.jpg', '.jpeg', '.bmp', '.tga', '.psd'] # Yes, we do supports .PSD files.

[[assets.importers]]
type = 'Transmission'
extensions = ['.obj', '.blend', '.fbx', '.gltf', '.dae', '.3ds']

[[assets.importers]]
type = 'Audio'
extensions = ['.mp3', '.wav', '.ogg', '.flac']
```

## Assets Workflow

### General Thoughts
The most important sub-command that comes with `crayon-cli` would be assets pre-processing. There are different requirement for asset to be processed in various situations.

1. The assets might be modified by artiest continuous, so it would be great if we store resource in formats which could producing and editing by authoring tools directly.

2. The most effecient format is dependent on platform and hardware devices. The assets might be converts to various formats based on the build target before packing into playable package.

3. The processing of assets from plain formats into runtime formats might causes heavily cpu consumption, and takes minutes for medium size project. By the same time, its a common requirement to edit and preview the effects on playable environment. So we should have some kind of mechanism to manage the asset processing incrementally.

4. Many software use path as identification of assets, it works fine before we refining the name or file structure of assets, and path itself is not platform independent yet. Its better to have a general GUID mapped to asset.

### How it Works
```sh
crayon-cli build
```

This CLI automatically imports assets and manages various kinds of additionla data about them for you, such as what import settings should be used to import the asset, below is a description of how this process works.

When you place an asset (_name_) in the specified `workspace::assets` folder, and runs `crayon-cli build`:

1. A meta-file _name_.meta.toml is created;
2. For every resource that this asset might produces, a universal-uniqued id is assigned to it;

And besides that, all the assets will be processed, converted to internal game-ready versions incrementally  in the `workspace::resources` folder.

### Meta-file

The UUID that `crayon-cli` assigns to each resource is stored inside the .meta.toml file alongside the asset file itself. This .meta file must stay with the asset file it relates to.

And also the meta files contain values for all the import settings, For a texture, this includes settings such as the `TextureWrap`, `TextureFilter` and `Compression` mode etc.. If you change the import settings for an asset, the asset will be re-imported according to your new settings with next `build` command.
 
* _Notes_ that .meta.toml files must match and stay with their respective asset files. If you move or rename an asset, you must move or rename the .meta.toml file to match, or you will lost all the references that points to it (a new GUID might be generated for it).
