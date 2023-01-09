struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) pos: vec2<f32>,
};

struct PushConstant {
	window_size: vec2<u32>,
	rect_pos: vec2<i32>,
	rect_size: vec2<u32>,
}

var<push_constant> pc: PushConstant;

@vertex
fn vs_main(
	@builtin(vertex_index) index: u32,
) -> VertexOutput {
	var out: VertexOutput;

	out.pos = vec2<f32>(0., 0.);

	var norm_pos = vec2<f32>(f32(pc.rect_pos.x)/f32(pc.window_size.x), f32(pc.rect_pos.y)/f32(pc.window_size.y));
	let norm_size = vec2<f32>(f32(pc.rect_size.x)/f32(pc.window_size.x), f32(pc.rect_size.y)/f32(pc.window_size.y));

	if index % u32(2) == u32(1) {
		norm_pos.y = norm_pos.y + norm_size.y;
		out.pos.y = 1.;

	}

	if index == u32(0) || index >= u32(4) {
		norm_pos.x = norm_pos.x + norm_size.x;
		out.pos.x = 1.;
	}

	out.clip_position = vec4<f32>(2. * norm_pos - 1., 1., 1.);

	return out;
}

// Fragment shader

@group(0)
@binding(0)
var tex: texture_storage_2d<rgba8unorm, read>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let dim = textureDimensions(tex);
	var pos = vec2<i32>(i32(in.pos.x * f32(dim.x)), i32(in.pos.y * f32(dim.y)));
	return textureLoad(tex, pos) + vec4<f32>(0., 0., 1., 1.);
}
