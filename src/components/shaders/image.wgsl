struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) pos: vec2<f32>,
};

@vertex
fn vs_main(
	@builtin(vertex_index) index: u32,
) -> VertexOutput {
	var out: VertexOutput;

	out.pos = vec2<f32>(0., 0.);

	if index % u32(2) == u32(1) {
		out.pos.y = 1.;
	}

	if index == u32(0) || index >= u32(4) {
		out.pos.x = 1.;
	}

	out.clip_position = vec4<f32>(2. * out.pos - 1., 1., 1.);

	return out;
}

// Fragment shader

@group(0)
@binding(0)
var tex: texture_storage_2d<rgba8unorm, read>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let dim = textureDimensions(tex);
	var pos = vec2<i32>(i32(in.pos.x * f32(dim.x)), i32((1. - in.pos.y) * f32(dim.y)));
	return textureLoad(tex, pos);
}
