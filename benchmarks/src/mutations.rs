use {scaffolding::plugin_prelude::*, std::hint::black_box, test::Bencher};

#[derive(Default)]
struct AppState {
    counter: i32,
}

#[derive(Clone)]
enum CommandType {
    Increment,
    Decrement,
}

#[derive(Default, Clone)]
struct Commands {
    ty: Option<CommandType>,
}
impl ExecutableArg for Commands {
    type Arg<'a> = Self;
    type Mutation = Self;

    fn from_world_and_env<'a>(_: &'a ImmutableWorld, _: &'a TypeMap) -> Self::Arg<'a> {
        Self::default()
    }
    fn build_mutation(self) -> Self::Mutation {
        self
    }
}
impl Mutation for Commands {
    type Reverse = Self;

    fn apply(self, world: &mut World) {
        if let Some(ty) = self.ty {
            let state: &mut AppState = world.get_state_mut();
            match ty {
                CommandType::Increment => state.counter += 1,
                CommandType::Decrement => state.counter -= 1,
            }
        }
    }
    fn build_reverse(&self, _: &World) -> Self::Reverse {
        let ty = self.ty.clone().map(|ty| match ty {
            CommandType::Increment => CommandType::Decrement,
            CommandType::Decrement => CommandType::Increment,
        });
        Self { ty }
    }
}

const ITERATIONS: u32 = 1;

#[bench]
fn scaffolding_mutations(b: &mut Bencher) {
    let mut world = World::new();
    world.add_state(AppState::default());
    b.iter(|| {
        for _ in 0..ITERATIONS {
            world.execute(
                |cmds: &mut Commands| {
                    cmds.ty = Some(CommandType::Increment);
                },
                None,
            );
        }
    });
}
#[bench]
fn scaffolding_dyn_mutations(b: &mut Bencher) {
    let mut world = World::new();
    world.add_state(AppState::default());
    b.iter(|| {
        for _ in 0..ITERATIONS {
            let (mutation, _) = black_box(world.execute_delayed_mutation(
                |cmds: &mut Commands| {
                    cmds.ty = Some(CommandType::Increment);
                },
                None,
            ));
            mutation.apply(&mut world);
        }
    });
}
#[bench]
fn normal_mutations(b: &mut Bencher) {
    let mut state = AppState { counter: 0 };
    b.iter(|| {
        for _ in 0..ITERATIONS {
            state.counter += black_box(1);
        }
    });
}
