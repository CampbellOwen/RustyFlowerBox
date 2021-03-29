use std::ops::Deref;

use cube::CUBE_VERTS;
use cube::{Vertex, CUBE_INDICES};

pub trait GraphicsDevice {
    fn set_vertex_buffer(&self, vertices: &[Vertex]);
    fn set_index_buffer(&self, vertices: &[u32]);
    fn draw(&self, num_vertices: u32);
}

pub mod cube;

pub fn upload_mesh(graphics_device: &Box<dyn GraphicsDevice>) {
    let graphics_device = graphics_device.deref();
    graphics_device.set_vertex_buffer(&CUBE_VERTS);
    graphics_device.set_index_buffer(&CUBE_INDICES);
}

pub fn draw(graphics_device: &Box<dyn GraphicsDevice>) {
    graphics_device.deref().draw(CUBE_VERTS.len() as u32);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
