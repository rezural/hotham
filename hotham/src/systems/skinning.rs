use cgmath::{Matrix4, SquareMatrix};
use legion::{system, world::SubWorld, Entity, EntityStore, IntoQuery};
use std::collections::HashMap;

use crate::{
    components::{Joint, Skin, TransformMatrix},
    resources::VulkanContext,
};

#[system]
#[read_component(Joint)]
#[read_component(TransformMatrix)]
#[read_component(Skin)]
pub(crate) fn skinning(world: &mut SubWorld, #[resource] vulkan_context: &VulkanContext) -> () {
    let mut joint_matrices: HashMap<Entity, Vec<Matrix4<f32>>> = HashMap::new();
    unsafe {
        let mut query = <(&TransformMatrix, &Joint)>::query();
        query.for_each_unchecked(world, |(transform_matrix, joint)| {
            let skeleton_root = joint.skeleton_root;
            let skeleton_root_entity = world.entry_ref(skeleton_root).unwrap();
            let inverse_transform = skeleton_root_entity
                .get_component::<TransformMatrix>()
                .unwrap()
                .0
                .invert()
                .unwrap();

            let joint_matrix = inverse_transform * transform_matrix.0 * joint.inverse_bind_matrix;
            let matrices = joint_matrices.entry(skeleton_root).or_default();
            matrices.push(joint_matrix);
        });
    }

    let mut query = <&Skin>::query();
    query.for_each_chunk(world, |chunk| {
        for (entity, skin) in chunk.into_iter_entities() {
            let matrices = joint_matrices.get(&entity).unwrap();
            let buffer = &skin.buffer;
            buffer
                .update(vulkan_context, matrices.as_ptr(), matrices.len())
                .unwrap();
        }
    });
}

#[cfg(test)]
mod tests {

    use crate::{
        buffer::Buffer,
        components::{Joint, Parent, Skin},
        resources::VulkanContext,
    };

    use super::*;
    use ash::version::DeviceV1_0;
    use cgmath::{vec3, Matrix4, SquareMatrix};
    use legion::{Resources, Schedule, World};

    #[test]
    pub fn test_skinning_system() {
        let mut world = World::default();
        let vulkan_context = VulkanContext::testing().unwrap();

        // Create the transform for the skin entity
        let translation = vec3(1.0, 2.0, 3.0);
        let root_transform_matrix = Matrix4::from_translation(translation);

        // Create a skin
        let inverse = root_transform_matrix.invert().unwrap();
        let joint_matrices = vec![inverse.clone(), inverse];
        let buffer = Buffer::new_from_vec(
            &vulkan_context,
            &joint_matrices,
            ash::vk::BufferUsageFlags::STORAGE_BUFFER,
        )
        .unwrap();

        let skin = Skin {
            joint_matrices,
            buffer,
        };

        // Now create the skin entity
        let skinned_entity = world.push((skin, TransformMatrix(root_transform_matrix)));

        // Create a child joint
        let child_joint = Joint {
            skeleton_root: skinned_entity,
            inverse_bind_matrix: Matrix4::identity(),
        };

        let child_translation = vec3(1.0, 0.0, 0.0);
        let matrix = Matrix4::from_translation(child_translation);
        let child = world.push((child_joint, TransformMatrix(matrix), Parent(skinned_entity)));

        // Create a grandchild joint
        let grandchild_joint = Joint {
            skeleton_root: skinned_entity,
            inverse_bind_matrix: Matrix4::identity(),
        };

        let grandchild_translation = vec3(1.0, 0.0, 0.0);
        let matrix = Matrix4::from_translation(grandchild_translation);
        let _grandchild = world.push((grandchild_joint, TransformMatrix(matrix), Parent(child)));

        let mut schedule = Schedule::builder().add_system(skinning_system()).build();
        let mut resources = Resources::default();
        resources.insert(vulkan_context.clone());
        schedule.execute(&mut world, &mut resources);

        let skin = world.entry(skinned_entity).unwrap();
        let skin = skin.get_component::<Skin>().unwrap();

        let matrices_from_buffer: &[Matrix4<f32>];

        unsafe {
            let memory = vulkan_context
                .device
                .map_memory(
                    skin.buffer.device_memory,
                    0,
                    ash::vk::WHOLE_SIZE,
                    ash::vk::MemoryMapFlags::empty(),
                )
                .unwrap();
            matrices_from_buffer = std::slice::from_raw_parts_mut(std::mem::transmute(memory), 2);
        }

        assert_eq!(matrices_from_buffer.len(), 2);
        for (from_buf, joint_matrices) in
            matrices_from_buffer.iter().zip(skin.joint_matrices.iter())
        {
            assert_ne!(*from_buf, Matrix4::identity());
            assert_ne!(from_buf, joint_matrices);
        }
    }
}