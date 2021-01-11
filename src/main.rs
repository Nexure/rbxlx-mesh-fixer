use core::panic;
use futures::future::Future;
use rbx_dom_weak::{
    types::{Ref, Variant},
    WeakDom,
};
use rbx_types::{CFrame, Matrix3, Vector3};
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
    sync::Arc,
};

mod utils;
use utils::{
    asset_downloader::download_asset, cframe::CFrameExt, mesh_reader::RobloxMesh, GenericError,
};

macro_rules! get_content {
    ($props:expr, $name:expr) => {
        match &$props[$name] {
            Variant::Content(prop) => {
                let raw_prop = prop.to_owned();
                raw_prop.into_string()
            }
            _ => panic!("Property has invalid type"),
        }
    };
}

macro_rules! get_size {
    ($props:expr, $name:expr) => {
        match &$props[$name] {
            Variant::Vector3(prop) => prop.to_owned(),
            _ => panic!("Property has invalid type"),
        }
    };
}

macro_rules! get_cframe {
    ($props:expr) => {
        match &$props["CFrame"] {
            Variant::CFrame(prop) => prop.to_owned(),
            _ => panic!("Property has invalid type"),
        }
    };
}

macro_rules! modify_property {
    ($props:expr, $prop:expr, $value:expr) => {
        if let Some(mut_prop) = $props.get_mut($prop) {
            *mut_prop = $value;
            // Variant::Content(rbx_types::Content::from(new_mesh.asset_id.clone()))
        } else {
            panic!(&format!("Failed to modify property {:?} on mesh", $prop))
        }
    };
}

struct CachedMesh {
    cframe: CFrame,
    mesh: RobloxMesh,
    asset_id: String,
    init_size: Vector3,
    size: Vector3,
}

fn open_rbx_place(input_path: String) -> Result<WeakDom, Box<dyn Error + 'static>> {
    let input_fp = Path::new(&input_path);
    let input_file = BufReader::new(File::open(input_fp)?);
    Ok(rbx_binary::from_reader_default(input_file)?)
}

fn save_rbx_place(output_path: String, dom: &WeakDom) -> Result<(), Box<dyn Error + 'static>> {
    let output_fp = Path::new(&output_path);
    let output_file = BufWriter::new(File::create(output_fp)?);
    // write_log(format!("{:#?}", dom));

    Ok(rbx_binary::to_writer_default(
        output_file,
        dom,
        dom.root().children(),
    )?)
}

fn write_log(string: String) -> Result<(), Box<dyn Error + 'static>> {
    let output_fp = Path::new("log.txt");
    let mut output_file = File::create(output_fp)?;
    output_file.write_all(string.as_bytes())?;
    Ok(())
}

fn get_children_recursive(children: &mut Vec<Ref>, dom: &WeakDom, referent: Ref) {
    let ref_inst = dom.get_by_ref(referent).expect("ref-inst");
    let ref_children = ref_inst.children();
    for child in ref_children {
        children.push(child.clone());
        get_children_recursive(children, dom, child.clone());
    }
}

fn filter_mesh_parts(dom: &WeakDom, refs: Vec<Ref>) -> Vec<Ref> {
    let mut mesh_parts = Vec::<Ref>::new();

    for referent in refs {
        let child = dom.get_by_ref(referent).expect("referent");
        if !child.properties.contains_key("TextureID") || !child.properties.contains_key("MeshId") {
            continue;
        }

        mesh_parts.push(referent);
    }

    mesh_parts
}

fn get_workspace_children(dom: &WeakDom) -> Vec<Ref> {
    let data_model = dom.root();
    let mut children = Vec::<Ref>::new();
    let workspace = *data_model
        .children()
        .iter()
        .find(|x| dom.get_by_ref(*x.to_owned()).unwrap().name == "Workspace")
        .expect("workspace");

    get_children_recursive(&mut children, dom, workspace);
    filter_mesh_parts(dom, children)
}

async fn download_meshs(dom: &WeakDom, refs: Vec<Ref>) -> Result<(), ()> {
    let mut handles = vec![];
    let master_semaphore = Arc::new(tokio::sync::Semaphore::new(4));

    for referent in refs {
        if let Some(child) = dom.get_by_ref(referent) {
            let mesh_id = get_content!(child.properties, "MeshId").clone();
            let semaphore = master_semaphore.clone();
            handles.push(tokio::spawn(async move {
                match semaphore.acquire().await {
                    Ok(_) => download_asset(mesh_id).await.is_ok(),
                    Err(_) => false,
                }
            }));
        } else {
            return Err(());
        }
    }

    let futures = futures::future::join_all(handles).await;
    if let Some(_) = futures.iter().find(|x| match x {
        Ok(item) => !item.clone(),
        Err(_) => true,
    }) {
        Err(())
    } else {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let input_path = std::env::args().nth(1).expect("input-path");
    let output_path = std::env::args().nth(2).expect("output-path");

    println!("Opening place..");
    let mut dom = open_rbx_place(input_path).expect("could not open place");
    println!("Getting children...");
    let children = get_workspace_children(&dom);
    println!("Found {:?} meshes", children.len());

    print!("Downloading meshes... ");
    if let Err(_) = download_meshs(&dom, children.clone()).await {
        eprintln!("Error downloading one or more assets");
        return;
    }
    println!("Done!");

    let mut textures = BTreeMap::<i32, CachedMesh>::new();
    for child_ref in children {
        let child = dom.get_by_ref_mut(child_ref).expect("workspace-child");
        let texture_id = get_content!(child.properties, "TextureID");
        let mesh_id = get_content!(child.properties, "MeshId");
        let init_size = get_size!(child.properties, "InitialSize");
        let size = get_size!(child.properties, "Size");
        let cframe = get_cframe!(child.properties);

        if texture_id.trim() == "" || mesh_id.trim() == "" {
            println!(
                "Instance {:?}, does not have valid textureId/meshId, skipping",
                child.name.clone()
            );
            continue;
        }

        let mesh = RobloxMesh::from_asset_id(mesh_id.clone())
            .await
            .expect("download-mesh");

        println!(
            "num_meshes={:?}, num_verts={:?}, num_faces={:?}, num_lod={:?}, num_bones={:?}, nts={:?}, nsd={:?}, stub={:?}, triangles={:?}, hash={:?}",
            mesh.header.num_meshes, mesh.header.num_verts, mesh.header.num_faces, mesh.header.num_lods, mesh.header.num_bones, mesh.header.name_table_size, mesh.header.num_skin_data, mesh.header.stub,
            mesh.triangles, mesh.hash
        );
        println!("bounding_box={:#?}", mesh.bounding_box);

        if textures.contains_key(&mesh.hash) {
            let new_mesh = &textures[&mesh.hash];

            modify_property!(
                child.properties,
                "MeshId",
                Variant::Content(rbx_types::Content::from(new_mesh.asset_id.clone()))
            );

            modify_property!(child.properties, "Size", Variant::Vector3(new_mesh.size));
            modify_property!(
                child.properties,
                "InitialSize",
                Variant::Vector3(new_mesh.init_size)
            );

            let rotation = mesh.calculate_rotation(&new_mesh.mesh);
            modify_property!(
                child.properties,
                "CFrame",
                Variant::CFrame(cframe.mult(CFrame::angles(0.0, rotation.y, 0.0)))
            );

            println!("rotation={:?}", rotation);
            println!(
                "Converting {:?} to {:?}",
                mesh_id,
                new_mesh.asset_id.clone()
            );
            println!("Id: {:?}", child.properties["MeshId"]);
        } else {
            textures.insert(
                mesh.hash,
                CachedMesh {
                    mesh: mesh,
                    asset_id: mesh_id.clone(),
                    cframe: cframe,
                    init_size: init_size,
                    size: size,
                },
            );
            println!("Cached {:?}", mesh_id);
        }

        println!("{:?}", child.name.clone());
    }

    println!("Done, converting it back to a place now.");
    save_rbx_place(output_path.clone(), &dom).expect("Failed to save place to file");
    println!("Finished, saved to path: {:?}", output_path);
}
