// type VtxShader = Box<dyn FnOnce() + Send + 'static>;
// type PixShader = Box<dyn FnOnce() + Send + 'static>;
//
//
// struct Varying {
// 	data: Vec<f32>,
// }
// impl Varying<T> {
// 	fn push(self, val: T) {
// 		self.data.push(val);
// 	}
// 	fn pop(self, val: T) {
// 		self.data.pop(val);
// 	}
// 	fn interpolate(self) {
//
// 	}
// }
//
// struct UnifiedShader {
// 	view: Mat4f,
// 	perspective_proj: Mat4f,
// 	ortho_proj: Mat4f,
// 	trianglepshaderdata:
// }
//
//
//
// impl UnifiedShader {
// 	fn new() -> UnifiedShader {
// 		UnifiedShader {
// 			view: Fmat4::identity(),
// 			perspective_proj: Fmat4::identity(),
// 			ortho_proj: Fmat4::identity(),
// 		}
// 	}
//
// 	fn run(self, vtx_shader: VtxShader, pix_shader: PixShader) {
// 		loop {
// 			vtx_shader();
// 			break;
// 		}
// 		loop {
// 			pix_shader();
// 			break;
// 		}
// 	}
// }