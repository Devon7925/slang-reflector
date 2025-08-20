use slang_reflector::{ProgramLayoutReflector, Downcast};

#[test]
fn multi_entrypoint() {
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
            .load_module(&"multi_entrypoint.slang")
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to load module: {:?}",
                    e.to_string()
                )
            });

        println!("Module loaded");

        for dependency in module.dependency_file_paths() {
            let dep_module = slang_session
                .load_module(dependency)
                .unwrap_or_else(|e| {
                    panic!(
                        "Failed to load dependency module: {:?}",
                        e.to_string()
                    )
                });

            println!("Dependency module loaded: {}", dependency);

            components.push(dep_module.downcast().clone());

            for entry_point in dep_module.entry_points() {
                components.push(entry_point.downcast().clone());
            }
        }
    }

    let program = slang_session
        .create_composite_component_type(components.as_slice())
        .unwrap();
    let linked_program = program.link().unwrap();

    let shader_reflection = linked_program.layout(0).unwrap();

    let multi_reflection = shader_reflection.reflect();

    assert_eq!(multi_reflection.entry_points[0].name, "fillBuffer", "First entrypoint was {}", multi_reflection.entry_points[0].name);
    assert_eq!(multi_reflection.entry_points[1].name, "fillBuffer2", "Second entrypoint was {}", multi_reflection.entry_points[1].name);
    assert_eq!(multi_reflection.entry_points[2].name, "fillBuffer3", "Third entrypoint was {}", multi_reflection.entry_points[2].name);
    assert_eq!(multi_reflection.entry_points[3].name, "fillBuffer4", "Fourth entrypoint was {}", multi_reflection.entry_points[3].name);
    assert_eq!(multi_reflection.entry_points[4].name, "fillBuffer5", "Fifth entrypoint was {}", multi_reflection.entry_points[4].name);
    assert_eq!(multi_reflection.entry_points[5].name, "fillBuffer6", "Sixth entrypoint was {}", multi_reflection.entry_points[5].name);
    assert_eq!(multi_reflection.entry_points[6].name, "fillBuffer7", "Seventh entrypoint was {}", multi_reflection.entry_points[6].name);
    assert_eq!(multi_reflection.entry_points[7].name, "fillBuffer8", "Eighth entrypoint was {}", multi_reflection.entry_points[7].name);
    assert_eq!(multi_reflection.entry_points[8].name, "fillBuffer9", "Ninth entrypoint was {}", multi_reflection.entry_points[8].name);
}