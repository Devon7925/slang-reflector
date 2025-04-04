pub use slang::{
    Blob, CompileTarget, CompilerOptions, ComponentType, Error, FileSystem, GlobalSession, Module,
    OptimizationLevel, Result, ScalarType, Session, SessionDesc, Stage, TargetDesc,
};
use slang::{ImageFormat, ParameterCategory, TypeKind, reflection::UserAttribute};

pub enum UserAttributeParameter {
    String(String),
    Int(i32),
    Float(f32),
}

pub struct UserAttributeReflection {
    pub name: String,
    pub parameters: Vec<UserAttributeParameter>,
}

pub enum TextureType {
    Dim1,
    Dim2,
    Dim3,
    Cube,
}

pub struct VariableReflection {
    pub name: String,
    pub reflection_type: BoundResource,
    pub user_attributes: Vec<UserAttributeReflection>,
}

pub enum BoundResource {
    Uniform {
        uniform_offset: usize,
        resource_result: VariableReflectionType,
    },
    StructuredBuffer(VariableReflectionType),
    Sampler,
    Texture {
        tex_type: TextureType,
        resource_result: VariableReflectionType,
        format: ImageFormat,
    },
}

pub enum VariableReflectionType {
    Struct(Vec<(String, VariableReflectionType)>),
    Scalar(slang::ScalarType),
    Vector(slang::ScalarType, usize),
}

pub struct EntrypointReflection {
    pub name: String,
    pub user_attributes: Vec<UserAttributeReflection>,
}

pub struct ProgramReflection {
    pub variables: Vec<VariableReflection>,
    pub entry_points: Vec<EntrypointReflection>,
}

pub trait ProgramLayoutReflector {
    fn reflect(&self) -> ProgramReflection;
}

impl ProgramLayoutReflector for slang::ProgramLayout {
    fn reflect(&self) -> ProgramReflection {
        let global_layout = self.global_params_type_layout();
        let var_reflection = if matches!(global_layout.kind(), TypeKind::ConstantBuffer) {
            global_layout.element_type_layout()
        } else {
            global_layout
        };

        let mut variables = Vec::new();

        for parameter in var_reflection.fields() {
            let reflection_type =
                bound_resource_from_slang_type(parameter.type_layout().ty().unwrap(), parameter);
            let user_attributes =
                parameter_user_attributes(parameter.variable().unwrap().user_attributes());
            variables.push(VariableReflection {
                name: parameter.variable().unwrap().name().to_string(),
                reflection_type,
                user_attributes,
            })
        }

        let mut entry_points = Vec::new();

        for entry_point in self.entry_points() {
            entry_points.push(EntrypointReflection {
                name: entry_point.name().to_string(),
                user_attributes: parameter_user_attributes(
                    entry_point.function().user_attributes(),
                ),
            })
        }

        ProgramReflection {
            variables,
            entry_points,
        }
    }
}

fn parameter_user_attributes<'a>(
    user_attributes: impl ExactSizeIterator<Item = &'a UserAttribute>,
) -> Vec<UserAttributeReflection> {
    let mut attributes = Vec::new();

    for attribute in user_attributes {
        let mut parameters = Vec::new();

        for i in 0..attribute.argument_count() {
            if let Some(string_arg) = attribute.argument_value_string(i) {
                parameters.push(UserAttributeParameter::String(
                    string_arg.trim_matches('"').to_string(),
                ))
            } else if let Some(int_arg) = attribute.argument_value_int(i) {
                parameters.push(UserAttributeParameter::Int(int_arg))
            } else if let Some(float_arg) = attribute.argument_value_float(i) {
                parameters.push(UserAttributeParameter::Float(float_arg));
            }
        }

        attributes.push(UserAttributeReflection {
            name: attribute.name().to_string(),
            parameters,
        })
    }

    attributes
}

fn bound_resource_from_slang_type(
    slang_type: &slang::reflection::Type,
    slang_layout: &slang::reflection::VariableLayout,
) -> BoundResource {
    match slang_layout.category() {
        slang::ParameterCategory::None => todo!(),
        slang::ParameterCategory::Mixed => todo!(),
        slang::ParameterCategory::ConstantBuffer => todo!(),
        slang::ParameterCategory::ShaderResource => match slang_type.resource_shape() {
            slang::ResourceShape::SlangResourceBaseShapeMask => todo!(),
            slang::ResourceShape::SlangResourceNone => todo!(),
            slang::ResourceShape::SlangTexture1d
            | slang::ResourceShape::SlangTexture2d
            | slang::ResourceShape::SlangTexture3d
            | slang::ResourceShape::SlangTextureCube => BoundResource::Texture {
                tex_type: resource_shape_to_tex_type(slang_type.resource_shape()),
                resource_result: reflection_type_from_slang_type(
                    slang_type.resource_result_type(),
                    Some(slang_layout.type_layout().element_var_layout()),
                ),
                format: slang_layout.image_format(),
            },
            slang::ResourceShape::SlangTextureBuffer => todo!(),
            slang::ResourceShape::SlangStructuredBuffer => {
                BoundResource::StructuredBuffer(reflection_type_from_slang_type(
                    slang_type.element_type(),
                    Some(slang_layout.type_layout().element_var_layout()),
                ))
            }
            slang::ResourceShape::SlangByteAddressBuffer => todo!(),
            slang::ResourceShape::SlangResourceUnknown => todo!(),
            slang::ResourceShape::SlangAccelerationStructure => todo!(),
            slang::ResourceShape::SlangTextureSubpass => todo!(),
            slang::ResourceShape::SlangResourceExtShapeMask => todo!(),
            slang::ResourceShape::SlangTextureFeedbackFlag => todo!(),
            slang::ResourceShape::SlangTextureShadowFlag => todo!(),
            slang::ResourceShape::SlangTextureArrayFlag => todo!(),
            slang::ResourceShape::SlangTextureMultisampleFlag => todo!(),
            slang::ResourceShape::SlangTexture1dArray => todo!(),
            slang::ResourceShape::SlangTexture2dArray => todo!(),
            slang::ResourceShape::SlangTextureCubeArray => todo!(),
            slang::ResourceShape::SlangTexture2dMultisample => todo!(),
            slang::ResourceShape::SlangTexture2dMultisampleArray => todo!(),
            slang::ResourceShape::SlangTextureSubpassMultisample => todo!(),
        },
        slang::ParameterCategory::UnorderedAccess => todo!(),
        slang::ParameterCategory::VaryingInput => todo!(),
        slang::ParameterCategory::VaryingOutput => todo!(),
        slang::ParameterCategory::SamplerState => BoundResource::Sampler,
        slang::ParameterCategory::Uniform => BoundResource::Uniform {
            uniform_offset: slang_layout.offset(ParameterCategory::Uniform),
            resource_result: reflection_type_from_slang_type(slang_type, Some(slang_layout)),
        },
        _ => panic!("Not recognized as valid top level type category"),
    }
}

fn reflection_type_from_slang_type(
    slang_type: &slang::reflection::Type,
    slang_layout: Option<&slang::reflection::VariableLayout>,
) -> VariableReflectionType {
    match slang_type.kind() {
        TypeKind::None => panic!("Unrecognized variable type"),
        TypeKind::Struct => {
            if slang_type.name() == "Atomic" {
                return reflection_type_from_slang_type(
                    slang_layout.unwrap().ty().unwrap(),
                    slang_layout,
                );
            }
            let layout_fields = slang_layout
                .iter()
                .flat_map(|l| l.type_layout().fields().map(Option::from))
                .chain(std::iter::repeat(None));
            VariableReflectionType::Struct(
                slang_type
                    .fields()
                    .zip(layout_fields)
                    .map(|(type_field, layout_field)| {
                        (
                            type_field.name().to_string(),
                            reflection_type_from_slang_type(type_field.ty(), layout_field),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        }
        TypeKind::Array => todo!(),
        TypeKind::Matrix => todo!(),
        TypeKind::Vector => VariableReflectionType::Vector(
            slang_type.element_type().scalar_type(),
            slang_type.element_count(),
        ),
        TypeKind::Scalar => VariableReflectionType::Scalar(slang_type.scalar_type()),
        TypeKind::ConstantBuffer => todo!(),
        TypeKind::TextureBuffer => todo!(),
        TypeKind::ShaderStorageBuffer => todo!(),
        TypeKind::ParameterBlock => todo!(),
        TypeKind::GenericTypeParameter => todo!(),
        TypeKind::Interface => todo!(),
        TypeKind::OutputStream => todo!(),
        TypeKind::MeshOutput => todo!(),
        TypeKind::Specialized => todo!(),
        TypeKind::Feedback => todo!(),
        TypeKind::Pointer => todo!(),
        TypeKind::DynamicResource => todo!(),
        TypeKind::Count => todo!(),
        _ => panic!("Unrecognized variable type"),
    }
}

fn resource_shape_to_tex_type(resource_shape: slang::ResourceShape) -> TextureType {
    match resource_shape {
        slang::ResourceShape::SlangResourceBaseShapeMask => todo!(),
        slang::ResourceShape::SlangResourceNone => todo!(),
        slang::ResourceShape::SlangTexture1d => TextureType::Dim1,
        slang::ResourceShape::SlangTexture2d => TextureType::Dim2,
        slang::ResourceShape::SlangTexture3d => TextureType::Dim3,
        slang::ResourceShape::SlangTextureCube => TextureType::Cube,
        slang::ResourceShape::SlangTextureBuffer => todo!(),
        slang::ResourceShape::SlangStructuredBuffer => todo!(),
        slang::ResourceShape::SlangByteAddressBuffer => todo!(),
        slang::ResourceShape::SlangResourceUnknown => todo!(),
        slang::ResourceShape::SlangAccelerationStructure => todo!(),
        slang::ResourceShape::SlangTextureSubpass => todo!(),
        slang::ResourceShape::SlangResourceExtShapeMask => todo!(),
        slang::ResourceShape::SlangTextureFeedbackFlag => todo!(),
        slang::ResourceShape::SlangTextureShadowFlag => todo!(),
        slang::ResourceShape::SlangTextureArrayFlag => todo!(),
        slang::ResourceShape::SlangTextureMultisampleFlag => todo!(),
        slang::ResourceShape::SlangTexture1dArray => todo!(),
        slang::ResourceShape::SlangTexture2dArray => todo!(),
        slang::ResourceShape::SlangTextureCubeArray => todo!(),
        slang::ResourceShape::SlangTexture2dMultisample => todo!(),
        slang::ResourceShape::SlangTexture2dMultisampleArray => todo!(),
        slang::ResourceShape::SlangTextureSubpassMultisample => todo!(),
    }
}
