@group(0)
@binding(0)
var tex: texture_storage_2d<rgba8unorm, read_write>;

@compute
@workgroup_size(10, 10, 1)
fn cpt_main(@builtin(global_invocation_id) gid: vec3<u32>) {
	let pos = vec2<i32>(i32(gid.x), i32(gid.y));
	textureStore(tex, pos,
		vec4<f32>(
			f32(pos.x % 100)/99.,
			f32(pos.y % 1000)/999.,
			0.,
			1.
		)
	);
}
