use super::{
    asset_downloader::download_asset,
    cframe::{Vector2Ext, Vector3Ext},
    GenericError,
};
use byteorder::{LittleEndian, ReadBytesExt};
use rbx_types::{Matrix3, Vector2, Vector3};
use std::io::{Cursor, Read};

#[derive(Debug, Clone)]
pub struct RobloxBoneWeights {
    pub bones: [u8; 4],
    pub weights: [u8; 4],
}

#[derive(Debug, Clone)]
pub struct RobloxMeshVertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub uv: Vector3,
    pub color: i32,
    pub weights: RobloxBoneWeights,
}

#[derive(Debug, Clone)]
pub struct RobloxMeshHeader {
    pub num_meshes: u16,
    pub num_verts: i32,
    pub num_faces: i32,
    pub num_lods: u16,
    pub num_bones: u16,
    pub num_skin_data: u16,
    pub name_table_size: i32,
    pub stub: u16,
}

#[derive(Debug, Clone)]
pub struct RobloxMeshBoundingBox {
    pub min_x: Vector3,
    pub max_x: Vector3,
    pub min_z: Vector3,
    pub max_z: Vector3,
}

#[derive(Debug, Clone)]
pub struct RobloxMeshBoundingBoxSize {
    pub min: Vector3,
    pub max: Vector3,
}

#[derive(Debug, Clone)]
pub struct RobloxMesh {
    pub header: RobloxMeshHeader,

    pub lods: Vec<i32>,
    pub faces: Vec<[i32; 3]>,
    pub vertices: Vec<RobloxMeshVertex>,

    // Custom fields
    pub bounding_box: RobloxMeshBoundingBox,
    pub bounding_box_size: RobloxMeshBoundingBoxSize,
    pub rotation: Vector3,
    pub triangles: i32,
    pub hash: i32,
}

macro_rules! check_set_min {
    ($pos:expr, $min:expr) => {
        if $pos < $min {
            $min = $pos
        }
    };
}

macro_rules! check_set_max {
    ($pos:expr, $max:expr) => {
        if $pos > $max {
            $max = $pos
        }
    };
}

impl RobloxMeshBoundingBox {}

impl RobloxMesh {
    fn default_vector() -> Vector3 {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn read_header(cursor: &mut Cursor<Vec<u8>>) -> Result<RobloxMeshHeader, GenericError> {
        let mut version: [u8; 13] = [0; 13];
        cursor.read(&mut version)?;

        assert_eq!(std::str::from_utf8(&version)?, "version 4.00\n");
        assert_eq!(cursor.read_i16::<LittleEndian>()?, 24);

        Ok(RobloxMeshHeader {
            num_meshes: cursor.read_u16::<LittleEndian>()?,
            num_verts: cursor.read_i32::<LittleEndian>()?,
            num_faces: cursor.read_i32::<LittleEndian>()?,
            num_lods: cursor.read_u16::<LittleEndian>()?,
            num_bones: cursor.read_u16::<LittleEndian>()?,
            name_table_size: cursor.read_i32::<LittleEndian>()?,
            num_skin_data: cursor.read_u16::<LittleEndian>()?,
            stub: cursor.read_u16::<LittleEndian>()?,
        })
    }

    fn read_vector3(cursor: &mut Cursor<Vec<u8>>) -> Result<Vector3, GenericError> {
        Ok(Vector3 {
            x: cursor.read_f32::<LittleEndian>()?,
            y: cursor.read_f32::<LittleEndian>()?,
            z: cursor.read_f32::<LittleEndian>()?,
        })
    }

    fn read_vert_weights(cursor: &mut Cursor<Vec<u8>>) -> Result<RobloxBoneWeights, GenericError> {
        let mut bones: [u8; 4] = [0; 4];
        let mut weights: [u8; 4] = [0; 4];

        cursor.read(&mut bones)?;
        cursor.read(&mut weights)?;

        Ok(RobloxBoneWeights {
            bones: bones,
            weights: weights,
        })
    }

    fn read_faces(
        header: &RobloxMeshHeader,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Vec<[i32; 3]>, GenericError> {
        let mut faces = Vec::<[i32; 3]>::with_capacity(header.num_faces as usize);
        for _ in 0..header.num_faces {
            faces.push([
                cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?,
            ]);
        }

        Ok(faces)
    }

    fn read_verts(
        header: &RobloxMeshHeader,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Vec<RobloxMeshVertex>, GenericError> {
        let mut verts = Vec::<RobloxMeshVertex>::with_capacity(header.num_verts as usize);
        for _ in 0..header.num_verts {
            verts.push(RobloxMeshVertex {
                position: RobloxMesh::read_vector3(cursor)?,
                normal: RobloxMesh::read_vector3(cursor)?,
                uv: RobloxMesh::read_vector3(cursor)?,
                color: cursor.read_i32::<LittleEndian>()?,
                weights: RobloxBoneWeights {
                    bones: [0; 4],
                    weights: [0; 4],
                },
            })
        }

        if header.num_bones > 0 {
            for x in 0..header.num_verts as usize {
                let mut vert = &mut verts[x];
                vert.weights = RobloxMesh::read_vert_weights(cursor)?;
            }
        }

        Ok(verts)
    }

    fn read_lods(
        header: &RobloxMeshHeader,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Vec<i32>, GenericError> {
        let mut lods = Vec::<i32>::with_capacity(header.num_lods as usize);
        for _ in 0..header.num_lods {
            lods.push(cursor.read_i32::<LittleEndian>()?);
        }

        Ok(lods)
    }

    /*
    fn get_most_right_point(&self) -> Vector3 {
        let max_x_vert = self.vertices[0].position.clone();

        for vertice in self.vertices.clone() {
            if max_x_vert.x > vertice.position.x {
                vertice = max_x_vert;
            }
        }
    }
    */

    fn get_vertex_and_remove(
        items: &mut Vec<RobloxMeshVertex>,
        f: &dyn Fn(RobloxMeshVertex, RobloxMeshVertex) -> bool,
    ) -> RobloxMeshVertex {
        let mut g_idx = 0;
        let mut g_item = items[g_idx].clone();
        for (idx, vertice) in items.iter().enumerate() {
            if f(vertice.clone(), g_item.clone()) {
                g_item = vertice.clone();
                g_idx = idx;
            }
        }

        items.remove(g_idx);
        g_item.clone()
    }

    fn calculate_bounding_box_size(&mut self) {
        let mut min = self.vertices[0].position;
        let mut max = self.vertices[0].position;

        for vertice in self.vertices.clone() {
            let pos = vertice.position;
            check_set_min!(pos.x, min.x);
            check_set_min!(pos.y, min.y);
            check_set_min!(pos.z, min.z);
            check_set_max!(pos.x, max.x);
            check_set_max!(pos.y, max.y);
            check_set_max!(pos.z, max.z);
        }

        self.bounding_box_size = RobloxMeshBoundingBoxSize { min, max };
    }

    fn caculate_bounding_box(&mut self) {
        // default the min and max to the first vertice position
        let mut vertices = self.vertices.clone();

        let min_x =
            RobloxMesh::get_vertex_and_remove(&mut vertices, &|vertice: RobloxMeshVertex,
                                                               prev_vertice: RobloxMeshVertex|
             -> bool {
                vertice.position.x < prev_vertice.position.x
            });

        let max_x =
            RobloxMesh::get_vertex_and_remove(&mut vertices, &|vertice: RobloxMeshVertex,
                                                               prev_vertice: RobloxMeshVertex|
             -> bool {
                vertice.position.x > prev_vertice.position.x
            });

        let min_z =
            RobloxMesh::get_vertex_and_remove(&mut vertices, &|vertice: RobloxMeshVertex,
                                                               prev_vertice: RobloxMeshVertex|
             -> bool {
                vertice.position.z < prev_vertice.position.z
            });

        let max_z =
            RobloxMesh::get_vertex_and_remove(&mut vertices, &|vertice: RobloxMeshVertex,
                                                               prev_vertice: RobloxMeshVertex|
             -> bool {
                vertice.position.z > prev_vertice.position.z
            });

        self.bounding_box = RobloxMeshBoundingBox {
            min_x: min_x.position,
            max_x: max_x.position,
            min_z: min_z.position,
            max_z: max_z.position,
        };
    }

    fn calculate_hash(&mut self) {
        let min = self.bounding_box_size.min.x
            + self.bounding_box_size.min.y
            + self.bounding_box_size.min.z;
        let max = self.bounding_box_size.max.x
            + self.bounding_box_size.max.y
            + self.bounding_box_size.max.z;
        self.hash = self.triangles + (min.abs() + max) as i32;
    }

    pub fn calculate_rotation(self, mesh2: &RobloxMesh) -> Vector3 {
        // let max_a = self.bounding_box.max;
        // let min_a = self.bounding_box.min;
        // let min_b = mesh2.bounding_box.min;
        // let max_b = mesh2.bounding_box.max;

        // max_a.x;
        // min_a.x;
        // max_a.z;
        // min_a.z;

        // let alpha = max_a.sub(min_a);
        // let alpha2 = Vector2::new(alpha.x, alpha.z).normalize();
        // let beta = max_b.sub(min_b);
        // let beta2 = Vector2::new(beta.x, beta.z).normalize();

        // let theta = alpha.dot(beta);
        // println!("dot: {:?}", theta);
        // println!("alpha: {:?}, beta: {:?}", alpha, beta);

        // let x_diff = max_2.x - max.x;
        // let z_diff = max_2.z - max.z;
        // let rot_y = (y_diff.atan2(x_diff) * 180.0 / std::f32::consts::PI) / 2.0;
        // let rot_y = z_diff.atan2(x_diff);

        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub async fn from_asset_id(asset_id: String) -> Result<RobloxMesh, GenericError> {
        let asset_data = &mut download_asset(asset_id).await?;
        RobloxMesh::from_cursor(asset_data)
    }

    pub fn from_cursor(cursor: &mut Cursor<Vec<u8>>) -> Result<RobloxMesh, GenericError> {
        let header = RobloxMesh::read_header(cursor)?;
        let mut mesh = RobloxMesh {
            header: header.clone(),
            vertices: RobloxMesh::read_verts(&header, cursor)?,
            faces: RobloxMesh::read_faces(&header, cursor)?,
            lods: RobloxMesh::read_lods(&header, cursor)?,

            // custom fields
            hash: 0,
            triangles: 0,
            rotation: RobloxMesh::default_vector(),
            bounding_box: RobloxMeshBoundingBox {
                min_x: RobloxMesh::default_vector(),
                max_x: RobloxMesh::default_vector(),
                min_z: RobloxMesh::default_vector(),
                max_z: RobloxMesh::default_vector(),
            },
            bounding_box_size: RobloxMeshBoundingBoxSize {
                min: RobloxMesh::default_vector(),
                max: RobloxMesh::default_vector(),
            },
        };

        if mesh.lods.len() > 1 {
            mesh.triangles = mesh.lods[1] - mesh.lods[0];
        }

        mesh.calculate_bounding_box_size();
        mesh.caculate_bounding_box();
        mesh.calculate_hash();

        Ok(mesh)
    }
}
//
