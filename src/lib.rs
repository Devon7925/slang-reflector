use std::collections::HashMap;

pub use shader_slang::{
    Blob, CompileTarget, CompilerOptions, ComponentType, Error, GlobalSession,
    ImageFormat, Module, OptimizationLevel, ResourceAccess, Result, ScalarType, Session,
    SessionDesc, Stage, TargetDesc, Downcast,
};
use shader_slang::{reflection::UserAttribute, ParameterCategory, ResourceShape, TypeKind};

#[cfg_attr(feature = "derive-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum UserAttributeParameter {
    String(String),
    Int(i32),
    Float(f32),
}

#[cfg_attr(feature = "derive-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct UserAttributeReflection {
    pub name: String,
    pub parameters: Vec<UserAttributeParameter>,
}

#[cfg_attr(feature = "derive-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum TextureType {
    Dim1,
    Dim2,
    Dim3,
    Cube,
}

#[cfg_attr(feature = "derive-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct VariableReflection {
    pub name: String,
    pub reflection_type: BoundParameter,
    pub user_attributes: Vec<UserAttributeReflection>,
}

#[cfg_attr(feature = "derive-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum BoundParameter {
    Uniform {
        uniform_offset: usize,
        resource_result: VariableReflectionType,
    },
    Resource {
        resource: BoundResource,
        binding_index: u32,
    },
}

#[cfg_attr(feature = "derive-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum BoundResource {
    StructuredBuffer {
        resource_result: VariableReflectionType,
        resource_access: ResourceAccess,
    },
    Sampler,
    Texture {
        tex_type: TextureType,
        resource_result: VariableReflectionType,
        format: ImageFormat,
        resource_access: ResourceAccess,
    },
}

#[cfg_attr(feature = "derive-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum VariableReflectionType {
    Struct {
        name: String, 
        props: Vec<(String, VariableReflectionType)>,
        user_attributes: Vec<UserAttributeReflection>,
    },
    Scalar(ScalarType),
    Vector(ScalarType, usize),
    Array(Box<VariableReflectionType>, usize),
}

fn get_scalar_size(scalar_type: &ScalarType) -> u32 {
    match scalar_type {
        ScalarType::Int8 | ScalarType::Uint8 => 1,
        ScalarType::Int16 | ScalarType::Uint16 | ScalarType::Float16 => 2,
        ScalarType::Int32 | ScalarType::Uint32 | ScalarType::Float32 => 4,
        ScalarType::Int64 | ScalarType::Uint64 | ScalarType::Float64 => 8,
        _ => panic!("Unrecognized scalar type"),
    }
}

impl VariableReflectionType {
    pub fn get_size(&self) -> u32 {
        match self {
            VariableReflectionType::Scalar(scalar_type) => get_scalar_size(scalar_type),
            VariableReflectionType::Vector(scalar_type, count) => {
                let count = count.next_power_of_two() as u32;
                count * get_scalar_size(scalar_type)
            }
            VariableReflectionType::Struct{ props, .. } => props
                .iter()
                .map(|(_, field_data)| field_data.get_size())
                .fold(0, |a, f| (a + f).div_ceil(f) * f),
            VariableReflectionType::Array(ty, count) => ty.get_size() * *count as u32,
        }
    }
}

#[cfg_attr(feature = "derive-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct EntrypointReflection {
    pub name: String,
    pub user_attributes: Vec<UserAttributeReflection>,
}

#[cfg_attr(feature = "derive-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct ProgramReflection {
    pub variables: Vec<VariableReflection>,
    pub entry_points: Vec<EntrypointReflection>,
    pub hashed_strings: HashMap<u32, String>,
}

pub trait ProgramLayoutReflector {
    fn reflect(&self) -> ProgramReflection;
}

impl ProgramLayoutReflector for shader_slang::reflection::Shader {
    fn reflect(&self) -> ProgramReflection {
        let global_layout = self.global_params_type_layout().unwrap();
        let var_reflection = if matches!(global_layout.kind(), TypeKind::ConstantBuffer) {
            global_layout.element_type_layout().unwrap()
        } else {
            global_layout
        };

        let mut variables = Vec::new();

        for parameter in var_reflection.fields() {
            let reflection_type =
                bound_resource_from_slang_type(parameter.type_layout().unwrap().ty().unwrap(), parameter);
            let user_attributes =
                parameter_user_attributes(parameter.variable().unwrap().user_attributes());
            variables.push(VariableReflection {
                name: parameter.variable().unwrap().name().unwrap().to_string(),
                reflection_type,
                user_attributes,
            })
        }

        let mut entry_points = Vec::new();

        for entry_point in self.entry_points() {
            entry_points.push(EntrypointReflection {
                name: entry_point.name().unwrap().to_string(),
                user_attributes: parameter_user_attributes(
                    entry_point.function().unwrap().user_attributes(),
                ),
            })
        }

        let hashed_strings = (0..self.hashed_string_count())
            .map(|i| self.hashed_string(i).unwrap().to_string())
            .map(|s| (shader_slang::reflection::compute_string_hash(s.as_str()), s))
            .collect();

        ProgramReflection {
            variables,
            entry_points,
            hashed_strings,
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
                parameters.push(UserAttributeParameter::String(string_arg.to_string()))
            } else if let Some(int_arg) = attribute.argument_value_int(i) {
                parameters.push(UserAttributeParameter::Int(int_arg))
            } else if let Some(float_arg) = attribute.argument_value_float(i) {
                parameters.push(UserAttributeParameter::Float(float_arg));
            }
        }

        attributes.push(UserAttributeReflection {
            name: attribute.name().unwrap().to_string(),
            parameters,
        })
    }

    attributes
}

fn bound_resource_from_slang_type(
    slang_type: &shader_slang::reflection::Type,
    slang_layout: &shader_slang::reflection::VariableLayout,
) -> BoundParameter {
    if matches!(slang_layout.category().unwrap(), ParameterCategory::Uniform) {
        return BoundParameter::Uniform {
            uniform_offset: slang_layout.offset(ParameterCategory::Uniform),
            resource_result: reflection_type_from_slang_type(
                slang_type,
                slang_layout.type_layout(),
            ),
        };
    }
    match slang_type.kind() {
        TypeKind::Resource => match slang_type.resource_shape() {
            ResourceShape::SlangTexture1d
            | ResourceShape::SlangTexture2d
            | ResourceShape::SlangTexture3d
            | ResourceShape::SlangTextureCube => BoundParameter::Resource {
                binding_index: slang_layout.binding_index(),
                resource: BoundResource::Texture {
                    tex_type: resource_shape_to_tex_type(slang_type.resource_shape()),
                    resource_result: reflection_type_from_slang_type(
                        slang_type.resource_result_type().unwrap(),
                        None,
                    ),
                    format: slang_layout.image_format(),
                    resource_access: slang_layout.type_layout().unwrap().resource_access().unwrap(),
                },
            },
            ResourceShape::SlangStructuredBuffer => BoundParameter::Resource {
                binding_index: slang_layout.binding_index(),
                resource: BoundResource::StructuredBuffer {
                    resource_result: reflection_type_from_slang_type(
                        slang_type.element_type().unwrap(),
                        slang_layout.type_layout().unwrap().element_type_layout(),
                    ),
                    resource_access: slang_layout.type_layout().unwrap().resource_access().unwrap(),
                },
            },
            rs => {
                panic!("{rs:?} resource shape not implemented for bound_resource_from_slang_type")
            }
        },
        TypeKind::SamplerState => BoundParameter::Resource {
            binding_index: slang_layout.binding_index(),
            resource: BoundResource::Sampler,
        },
        ty => panic!(
            "{ty:?} not recognized as valid top level type category for object of type {:?}",
            slang_type.kind()
        ),
    }
}

fn reflection_type_from_slang_type(
    slang_type: &shader_slang::reflection::Type,
    slang_layout: Option<&shader_slang::reflection::TypeLayout>,
) -> VariableReflectionType {
    match slang_type.kind() {
        TypeKind::None => panic!("Unrecognized variable type"),
        TypeKind::Struct => {
            if slang_type.name().unwrap() == "Atomic" {
                return reflection_type_from_slang_type(
                    slang_layout.unwrap().ty().unwrap(),
                    slang_layout,
                );
            }
            let layout_fields = slang_layout
                .iter()
                .flat_map(|l| l.fields().map(Option::from))
                .chain(std::iter::repeat(None));

            let user_attributes = parameter_user_attributes(slang_type.user_attributes());
            VariableReflectionType::Struct {
                name: slang_type.name().unwrap().to_string(),
                props: slang_type
                    .fields()
                    .zip(layout_fields)
                    .map(|(type_field, layout_field)| {
                        (
                            type_field.name().unwrap().to_string(),
                            reflection_type_from_slang_type(
                                type_field.ty().unwrap(),
                                layout_field.map(|l| l.type_layout().unwrap()),
                            ),
                        )
                    })
                    .collect::<Vec<_>>(),
                user_attributes,
            }
        }
        TypeKind::Array => VariableReflectionType::Array(
            Box::new(reflection_type_from_slang_type(
                slang_type.element_type().unwrap(),
                slang_layout.map(|l| l.element_type_layout().unwrap()),
            )),
            slang_type.element_count(),
        ),
        TypeKind::Matrix => todo!(),
        TypeKind::Vector => VariableReflectionType::Vector(
            slang_type.element_type().unwrap().scalar_type(),
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

fn resource_shape_to_tex_type(resource_shape: ResourceShape) -> TextureType {
    match resource_shape {
        ResourceShape::SlangTexture1d => TextureType::Dim1,
        ResourceShape::SlangTexture2d => TextureType::Dim2,
        ResourceShape::SlangTexture3d => TextureType::Dim3,
        ResourceShape::SlangTextureCube => TextureType::Cube,
        _ => todo!(),
    }
}
