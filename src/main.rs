use bevy::prelude::*;
use roto::{library, runtime::context::{NoCtx, OptCtx}, value::Val};

#[derive(Component, Debug, Clone, PartialEq)]
struct OurTarget;

#[derive(Component, Debug, Clone, PartialEq)]
struct ComponentA(f32);

#[derive(Component, Debug, Clone, PartialEq)]
struct ComponentB;

#[derive(Clone, Debug, PartialEq)]
enum InsertComponent {
    A(ComponentA),
    B(ComponentB),
}

#[derive(Clone, Debug, PartialEq)]
enum RemoveComponent {
    A,
    B,
}

#[derive(Clone, Debug, PartialEq)]
enum RotoInstruction {
    Insert(Entity, InsertComponent),
    Remove(Entity, RemoveComponent),
}

#[derive(Clone, EntityEvent, Debug)]
struct AnEvent(Entity);

#[derive(Resource)]
struct MyRoto {
    blob: roto::Blob<NoCtx>, // TOOD: `NoCtx` ?
}

fn main() {
    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    let default_plugin = DefaultPlugins
        .set(AssetPlugin {
            file_path: asset_root_path.clone(),
            ..Default::default()
        })
        .build();
    let roto_lib = library!(
        #[clone] type RotoInstruction = Val<RotoInstruction>;
        #[clone] type InsertComponent = Val<InsertComponent>;
        #[clone] type ComponentA = Val<ComponentA>;
        #[clone] type ComponentB = Val<ComponentB>;
        #[clone] type RemoveComponent = Val<RemoveComponent>;
        #[copy] type Entity = Val<Entity>;

        impl Val<RotoInstruction> {
            fn insert_a(e: Val<Entity>, n: f32) -> Self {
                Val(RotoInstruction::Insert(e.0, InsertComponent::A(ComponentA(n))))
            }
            fn insert_b(e: Val<Entity>) -> Self {
                Val(RotoInstruction::Insert(e.0, InsertComponent::B(ComponentB)))
            }
            fn remove_a(e: Val<Entity>) -> Self {
                Val(RotoInstruction::Remove(e.0, RemoveComponent::A))
            }
            fn remove_b(e: Val<Entity>) -> Self {
                Val(RotoInstruction::Remove(e.0, RemoveComponent::B))
            }
        }

        impl u32 {
            fn into_f32(self) -> f32 {
                self as f32
            }
        }
    );
    let mut roto_rt = roto::Runtime::from_lib(roto_lib).unwrap();
    
    let roto_script_path = asset_root_path + "/test.roto";
    App::new()
        .insert_resource(MyRoto {
            blob: roto_rt.compile(roto_script_path.as_str()).unwrap(),
        })
        .add_plugins(default_plugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_key_input,inspect))
        .run();
}

fn setup(mut commands: Commands) {
    let id = commands.spawn((OurTarget,)).id();

    commands.entity(id).observe(move |ev:On<AnEvent>, mut commands: Commands, mut cnt: Local<usize>, mut roto: ResMut<MyRoto>| {
        let fn_handle_an_event = roto.blob.get_function::<fn(Val<Entity>, u32) -> roto::value::List<Val<RotoInstruction>>>("handle_an_event").unwrap();

        let instructions = fn_handle_an_event.call(Val::from(ev.0), *cnt as u32);

        for instruction in instructions.into_iter() {
            match instruction.0 {
                RotoInstruction::Insert(entity, insert_component) => {
                    match insert_component {
                        InsertComponent::A(comp_a) => {
                            commands.entity(entity).try_insert(comp_a.clone());
                        }
                        InsertComponent::B(comp_b) => {
                            commands.entity(entity).try_insert(comp_b.clone());
                        }
                    }
                }
                RotoInstruction::Remove(entity, remove_component) => {
                    match remove_component {
                        RemoveComponent::A => {
                            commands.entity(entity).try_remove::<ComponentA>();
                        }
                        RemoveComponent::B => {
                            commands.entity(entity).try_remove::<ComponentB>();
                        }
                    }
                }
            }
        }

        *cnt += 1;
    });
    info!("id: {:?} is spawned", id);
}

fn inspect(q: Query<(Entity, Option<&ComponentA>, Option<&ComponentB>), With<OurTarget>>) {
    for (entity, comp_a, comp_b) in q.iter() {
        info!(
            "Entity {:?} has ComponentA: {:?}, ComponentB: {:?}",
            entity, comp_a, comp_b
        );
    }
}


fn handle_key_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_our_targets: Query<(Entity, &OurTarget)>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        for (entity, _target) in q_our_targets.iter() {
            info!("Key A pressed, sending event to entity {:?}", entity);
            commands.entity(entity).trigger(AnEvent);
        }
    }
}
