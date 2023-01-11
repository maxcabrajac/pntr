@group(0)
@binding(0)
var tex: texture_storage_2d<rgba8unorm, read_write>;

var<push_constant> clear_color: vec3<f32>;

@compute
@workgroup_size(8, 8, 1)
fn clear(@builtin(global_invocation_id) gid: vec3<u32>) {
	let pos = vec2<i32>(gid.xy);
	let dims = textureDimensions(tex);

	if pos.x > dims.x || pos.y > dims.y {
		return;
	}

	textureStore(tex, pos, vec4<f32>(clear_color, 1.));
}


struct DrawInput {
	mouse: vec2<i32>,
	brush_rad: u32
}
var<push_constant> in: DrawInput;

@compute
@workgroup_size(8, 8, 1)
fn draw(@builtin(global_invocation_id) gid: vec3<u32>) {
	let pos = vec2<i32>(gid.xy) + in.mouse - vec2<i32>(vec2<u32>(in.brush_rad, in.brush_rad));
	let dims = textureDimensions(tex);
	if 0 > pos.x || pos.x > dims.x || 0 > pos.y || pos.y > dims.y {
		return;
	}

	if distance(vec2<f32>(pos), vec2<f32>(in.mouse)) < f32(in.brush_rad) {
		let color = vec4(1., 1., 1., 1.);
		textureStore(tex, pos, color);
	}

}
