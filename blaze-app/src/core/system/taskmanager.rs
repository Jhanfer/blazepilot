// use std::sync::{Mutex, OnceLock};
// use std::collections::HashMap;
// use std::time::{Instant, Duration};

// use crate::{TaskStatusSlint, TaskProgressSlint, TaskKindSlint};
use crate::core::files::motor::TaskType;

// #[derive(Clone, Debug)]
// pub struct TaskProgress {
//     pub task_id: u64,
//     pub text: String,
//     pub progress: f32,
//     pub status: TaskStatusSlint,
//     pub task_kind: TaskKindSlint,
//     pub finished_at: Option<Instant>
// }

#[derive(Debug, Clone)]
pub enum TaskMessage {
    Started {
        task_id: u64,
        text: String,
        task_type: TaskType,
    },
    Progress {
        task_id: u64,
        progress: f32,
        text: String,
        task_type: TaskType,
    },
    Finished {
        task_id: u64,
        success: bool,
        task_type: TaskType,
        text: String
    }
}

// pub struct TaskManager {
//     tasks: Mutex<HashMap<u64, TaskProgress>>,
// }


// static TASK_MANAGER: OnceLock<TaskManager> = OnceLock::new();


// impl TaskManager {
//     pub fn global() -> &'static Self {
//         TASK_MANAGER.get_or_init(|| TaskManager {
//             tasks: Mutex::new(HashMap::new()),
//         })
//     }

//     pub fn start_task(&self, task_id: u64, text: String, task_kind: TaskType) {
//         let mut tasks = self.tasks.lock().unwrap();
//         let kind: TaskKindSlint; 

//         match task_kind {
//             TaskType::CopyPaste | TaskType::CutPaste => kind = TaskKindSlint::Pasting,
//             TaskType::FileLoading => kind = TaskKindSlint::FileLoading,
//             TaskType::Delete | TaskType::MoveTrash =>  kind = TaskKindSlint::Deleting
//         };

//         tasks.insert(task_id, TaskProgress { 
//             progress: 0.0, 
//             status: TaskStatusSlint::Running, 
//             task_id: task_id, 
//             text: text.into(),
//             task_kind: kind,
//             finished_at: None,
//         });
//     }

//     pub fn update_task(&self, task_id: u64, progress: f32, text: String, task_kind: TaskType) {
//         let mut tasks = self.tasks.lock().unwrap();
//         if let Some(task) = tasks.get_mut(&task_id) {
//             task.progress = progress;
//             task.text = text;
//         }
//     }

//     pub fn finish_task(&self, task_id: u64, success: bool, task_kind: TaskType) {
//         let mut tasks = self.tasks.lock().unwrap();
//         if let Some(task) = tasks.get_mut(&task_id) {
//             task.status = if success {
//                 TaskStatusSlint::FinishedSuccess
//             } else {
//                 TaskStatusSlint::FinishedError
//             };
//             task.finished_at = Some(Instant::now());
//             task.progress = 1.0;
//         }
//     }

//     pub fn get_tasks(&self) -> Vec<TaskProgressSlint> {
//         let mut tasks = self.tasks.lock().unwrap();

//         tasks.retain(|_, task|{
//             if let Some(finished_at) = task.finished_at {
//                 finished_at.elapsed() < Duration::from_secs(3) 
//             } else {
//                 true
//             }
//         });

//         tasks.values()
//             .map(|t| TaskProgressSlint {
//                 task_id: t.task_id as i32,
//                 text: t.text.clone().into(),
//                 progress: t.progress,
//                 status: t.status,
//                 task_kind: t.task_kind,
//             }).collect()
//     }

//     pub fn process_message(&self, message: TaskMessage) {
//         match message {
//             TaskMessage::Started { task_id, text, task_type } => {
//                 self.start_task(task_id, text, task_type);
//             },
//             TaskMessage::Progress { task_id, progress, text, task_type } => {
//                 self.update_task(task_id, progress, text, task_type);
//             },
//             TaskMessage::Finished { task_id, success,task_type, text } => {
//                 self.finish_task(task_id, success, task_type);
//             }
//         }
//     }
// }