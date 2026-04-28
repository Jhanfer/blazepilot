// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.



use std::{collections::HashMap, sync::{Mutex, OnceLock}, time::{Duration, Instant}};
use uuid::Uuid;
use crate::{core::files::motor::TaskType, utils::channel_pool::with_channel_pool};

#[derive(Clone, Debug)]
pub enum TaskStatus {
    Running,
    FinishedSuccess,
    FinishedError
}


#[derive(Clone, Debug)]
pub struct TaskProgress {
    pub task_id: u64,
    pub text: String,
    pub progress: f32,
    pub status: TaskStatus,
    pub task_kind: TaskType,
    pub finished_at: Option<Instant>
}

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
        #[allow(unused)]
        text: String
    }
}

pub struct TaskManager {
    tasks: Mutex<HashMap<u64, TaskProgress>>,
}


static TASK_MANAGER: OnceLock<TaskManager> = OnceLock::new();


impl TaskManager {
    pub fn global() -> &'static Self {
        TASK_MANAGER.get_or_init(|| TaskManager {
            tasks: Mutex::new(HashMap::new()),
        })
    }

    pub fn start_task(&self, task_id: u64, text: String, task_kind: TaskType) {
        let mut tasks = self.tasks.lock().unwrap();

        tasks.insert(task_id, TaskProgress { 
            progress: 0.0, 
            status: TaskStatus::Running, 
            task_id: task_id, 
            text: text.into(),
            task_kind: task_kind,
            finished_at: None,
        });
    }

    pub fn update_task(&self, task_id: u64, progress: f32, text: String, _task_kind: TaskType) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(&task_id) {
            task.progress = progress;
            task.text = text;
        }
    }

    pub fn finish_task(&self, task_id: u64, success: bool, _task_kind: TaskType) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(&task_id) {
            task.status = if success {
                TaskStatus::FinishedSuccess
            } else {
                TaskStatus::FinishedError
            };
            task.finished_at = Some(Instant::now());
            task.progress = 1.0;
        }
    }

    pub fn get_tasks(&self) -> Vec<TaskProgress> {
        let mut tasks = self.tasks.lock().unwrap();

        tasks.retain(|_, task|{
            if let Some(finished_at) = task.finished_at {
                finished_at.elapsed() < Duration::from_secs(3) 
            } else {
                true
            }
        });

        tasks.values()
            .map(|t| TaskProgress {
                task_id: t.task_id,
                text: t.text.clone().into(),
                progress: t.progress,
                status: t.status.clone(),
                task_kind: t.task_kind.clone(),
                finished_at: t.finished_at,
            }).collect()
    }

    pub fn process_message(&self, active_id: Uuid) {

        let tasks_messages: Vec<TaskMessage> = with_channel_pool(|pool|{
            let mut msgs = Vec::new();
            pool.process_task_messages(active_id, |msg|{
                msgs.push(msg);
                true
            });
            msgs
        });

        for msg in tasks_messages {
            match msg {
                TaskMessage::Started { task_id, text, task_type } => {
                    self.start_task(task_id, text, task_type);
                },
                TaskMessage::Progress { task_id, progress, text, task_type } => {
                    self.update_task(task_id, progress, text, task_type);
                },
                TaskMessage::Finished { task_id, success,task_type, text:_ } => {
                    self.finish_task(task_id, success, task_type);
                }
            }
        }

    }
}