// Vertex shader

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) pos: vec3<f32>,
};

@vertex
fn vs_main(
	@builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
	var out: VertexOutput;
	let x = f32(1 - i32(in_vertex_index)) * 0.5;
	let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;

	out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

	if in_vertex_index == u32(0) {
		out.pos = vec3<f32>(1., 0., 0.);
	} else if in_vertex_index == u32(1) {
		out.pos = vec3<f32>(0., 1., 0.);
	} else if in_vertex_index == u32(2) {
		out.pos = vec3<f32>(0., 0., 1.);
	}

	let temp = abs(out.clip_position);
	out.pos = vec3<f32>(temp.xyz);

   return out;
}

// Fragment shader


let CONT = vec4<f32>(1.0, 1.0, 1.0, 1.0);

fn dentroDoSierpinski(posIn: vec3<f32>, recIn: i32) -> bool {
	var rec = recIn;
	var pos = posIn;
	while rec > 0 {
		let x: bool = pos.x < .5;
		let y: bool = pos.y < .5;
		let z: bool = pos.z < .5;

		if x && y && z {
			return false;
		}

		pos = 2. * pos;
		pos.x = pos.x - f32(!x);
		pos.y = pos.y - f32(!y);
		pos.z = pos.z - f32(!z);

		rec = rec - 1;
	}
	return true;
}

//Available for fragment and compute
@group(0)
@binding(0)
var tex: texture_storage_2d<rgba8unorm, read_write>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return textureLoad(tex, vec2<i32>(i32(in.pos.x*100.), 0));
}

@compute
@workgroup_size(1)
fn compute_main(@builtin(global_invocation_id) gid: vec3<u32>) {
	let pos = vec2<i32>(i32(gid.x), i32(gid.y));
	textureStore(tex, pos, vec4<f32>(f32(i32(gid.x) % 10)/9., 0., 0., 1.));
}
