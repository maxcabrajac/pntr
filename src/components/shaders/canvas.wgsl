@group(0) @binding(0)
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

fn inside_circle(center: vec2<f32>, radius: f32, p: vec2<f32>) -> bool {
	return distance(center, p) <= radius;
}

struct DrawInput {
	mouse: vec2<i32>,
	brush_rad: u32
}
var<push_constant> point_in: DrawInput;

@compute
@workgroup_size(8, 8, 1)
fn draw_point(@builtin(global_invocation_id) gid: vec3<u32>) {
	let pos = vec2<i32>(gid.xy) + point_in.mouse - vec2<i32>(vec2<u32>(point_in.brush_rad, point_in.brush_rad));
	let dims = textureDimensions(tex);
	if 0 > pos.x || pos.x > dims.x || 0 > pos.y || pos.y > dims.y {
		return;
	}

	if inside_circle(vec2<f32>(point_in.mouse), f32(point_in.brush_rad), vec2<f32>(pos)) {
		let color = vec4<f32>(1., 1., 1., 1.);
		textureStore(tex, pos, color);
	}
}

fn scalar_projection(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
	let v1 = b - a;
	let v2 = c - a;

	return dot(v1, v2) / length(v1);
}

fn scalar_rejection(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
	let v1 = b - a;
	let v2 = c - a;

	return (v1.y * v2.x - v1.x * v2.y) / length(v1);
}

fn inside_line(a: vec2<f32>, b: vec2<f32>, width: f32, p: vec2<f32>) -> bool {
	let proj = scalar_projection(a, b, p);
	let rej = scalar_rejection(a, b, p);

	return abs(rej) <= width && proj >= 0. && proj <= length(b - a);
}

struct LineInput {
	reference_point: vec2<i32>,
	line_start_index: u32,
	line_end_index: u32,

	brush_rad: u32,
}

var<push_constant> line_in: LineInput;

@group(1) @binding(0)
var<storage, read> points: array<vec2<i32>>;

@compute
@workgroup_size(8, 8, 1)
fn draw_line(@builtin(global_invocation_id) gid: vec3<u32>) {
	let pos = vec2<i32>(gid.xy) + line_in.reference_point;
	let dims = textureDimensions(tex);
	if 0 > pos.x || pos.x > dims.x || 0 > pos.y || pos.y > dims.y {
		return;
	}

	let r = f32(line_in.brush_rad);

	var i = line_in.line_start_index;
	var flag = false;
	while i < line_in.line_end_index - u32(1) {
		let a = vec2<f32>(points[i]);
		let b = vec2<f32>(points[i+u32(1)]);
		flag = inside_circle(a, r, vec2<f32>(pos)) ||
				inside_line(a, b, r, vec2<f32>(pos)) ||
				inside_circle(b, r, vec2<f32>(pos));
		if flag { break; }
		i = i + u32(1);
	}

	if flag {
		let color = vec4<f32>(1., 1., 1., 1.);
		textureStore(tex, pos, color);
	}
}

