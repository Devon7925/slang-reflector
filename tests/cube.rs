use shader_slang::Downcast;
use slang_reflector::ProgramLayoutReflector;

#[test]
fn cube() {
    let global_slang_session = slang_reflector::GlobalSession::new().unwrap();

    let session_options = slang_reflector::CompilerOptions::default()
        .optimization(slang_reflector::OptimizationLevel::High)
        .matrix_layout_row(true);

    let target_desc = slang_reflector::TargetDesc::default()
        .format(slang_reflector::CompileTarget::Wgsl)
        .profile(global_slang_session.find_profile("spirv_1_6"));

    let targets = [target_desc];

    let search_paths = vec!["tests"];

    let search_paths = search_paths
        .into_iter()
        .map(std::ffi::CString::new)
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    let search_paths = search_paths.iter().map(|p| p.as_ptr()).collect::<Vec<_>>();

    let session_desc = slang_reflector::SessionDesc::default()
        .search_paths(&search_paths)
        .targets(&targets)
        .options(&session_options);

    let Some(slang_session) = global_slang_session.create_session(&session_desc) else {
        panic!("Failed to create slang session");
    };

    println!("Session loaded");

    let mut components: Vec<slang_reflector::ComponentType> = vec![];
    {
        let module = slang_session
            .load_module(&"cube.slang")
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to load module: {:?}",
                    e.to_string()
                )
            });

        println!("Module loaded");

        components.push(module.downcast().clone());

        for entry_point in module.entry_points() {
            components.push(entry_point.downcast().clone());
        }
    }

    let program = slang_session
        .create_composite_component_type(components.as_slice())
        .unwrap();
    let linked_program = program.link().unwrap();

    let shader_reflection = linked_program.layout(0).unwrap();

    shader_reflection.reflect();
}