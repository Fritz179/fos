use std::rc::{Weak, Rc};

use crate::{fc::table::Table, Process, Pid, Proc, Root, ROOT};

#[derive(Debug)]
pub struct Spawner {
    processes: Table<Weak<dyn Process>>,
}

impl Spawner {
    pub const fn new() -> Self {
        Spawner {
            processes: Table::new(),
        }
    }

    pub fn spawn<Child: Process + 'static>(&self) -> Rc<Child> {
        let processes = &self.processes;

        let child_pid = processes.next_free() as Pid;
        let child_proc = Proc::new(child_pid);

        // self.fs.add_pid(child_pid);

        let child = Rc::new(Child::new(child_proc));
        let id = processes.add(Rc::downgrade(&child) as Weak<dyn Process>);

        assert_eq!(child_pid, id as u32);


        child
    }

    pub fn spawn_root() -> Rc<Root> {
        let root = Rc::new(Root::new(Proc::new(0)));

        let spawner = &root.spawner;
        let id = spawner.processes.add(Rc::downgrade(&root) as Weak<dyn Process>);
        assert_eq!(id, 0);

        root
    }
}

impl Proc {
    pub fn spawn<Child: Process + 'static>(&self) -> Rc<Child> {
        let mut children = self.children.borrow_mut();
    
        let child = ROOT.spawner.spawn::<Child>();
        let child_clone = Rc::clone(&child);
        children.push(child_clone);
    
        child
    }
}