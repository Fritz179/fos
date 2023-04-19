use std::rc::{Weak, Rc};

use crate::{fc::table::Table, Process, Pid, Proc, Root};

#[derive(Debug)]
pub struct Spawner {
    processes: Table<Weak<dyn Process>>,
}

impl Spawner {
    pub fn new() -> Self {
        Spawner {
            processes: Table::new(),
        }
    }

    pub fn spawn<Child: Process + 'static>(&self, root: &Rc<Root>) -> (Rc<Child>, Pid) {
        let processes = &self.processes;

        let child_pid = processes.next_free() as Pid;
        let child_proc = Proc::new(child_pid, Rc::clone(root));

        // self.fs.add_pid(child_pid);

        child_proc.open(); // stdin
        child_proc.open(); // stdout
        child_proc.open(); // stderr

        let child = Rc::new(Child::new(child_proc));
        let id = processes.add(Rc::downgrade(&child) as Weak<dyn Process>);

        assert_eq!(child_pid, id as u32);


        return (child, child_pid);
    }

    pub fn spawn_root() -> Rc<Root> {

        let root = Rc::new_cyclic(|weak_root| {
            let weak_root_ptr = weak_root.as_ptr();

            // Safety: the Weak becomes an Rc, Proc stores internally Rc<Root> but doesn't use it in the Proc::new().
            let root_ptr = unsafe {
                Rc::from_raw(weak_root_ptr)
            };

            // Rc<Root> in not yet valid.
            let proc = Proc::new(0, root_ptr);

            // Making Rc<Root> valid
            Root::new(proc)
        });
        // Rc<Root is now valid>

        // the proc now has a valid Rc<Root>
        root.proc.open(); // stdin
        root.proc.open(); // stdout
        root.proc.open(); // stderr

        let spawner = &root.spawner;
        spawner.processes.add(Rc::downgrade(&root) as Weak<dyn Process>);

        root.main();

        root
    }
}

impl Proc {
    pub fn spawn<Child: Process + 'static>(&self) -> (Rc<Child>, Pid) {
        let mut children = self.children.borrow_mut();
    
        let (child, pid) = self.root.spawner.spawn::<Child>(&self.root);
        let child_clone = Rc::clone(&child);
        children.push(child_clone);
    
        return (child, pid);
    }
}