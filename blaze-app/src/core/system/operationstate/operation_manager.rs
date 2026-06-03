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

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::OnceLock;

use crate::core::runtime::bus_structs::FileOperation;
use crate::core::runtime::event_bus::Dispatcher;
use crate::core::system::operationstate::undo_record::UndoRecord;

static OPERATION_HISTORY: OnceLock<Mutex<OperationHistoryManager>> = OnceLock::new();

fn operation_history() -> &'static Mutex<OperationHistoryManager> {
    OPERATION_HISTORY.get_or_init(|| Mutex::new(OperationHistoryManager::new(50)))
}

pub fn with_history<F, R>(f: F) -> R
where
    F: FnOnce(&mut OperationHistoryManager) -> R,
{
    f(&mut operation_history().lock())
}

pub struct OperationHistoryManager {
    history: VecDeque<UndoRecord>,
    max_size: usize,
}

impl OperationHistoryManager {
    pub fn new(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn history(&self) -> &VecDeque<UndoRecord> {
        &self.history
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn push(&mut self, record: UndoRecord) {
        if self.history.len() >= self.max_size {
            self.history.pop_front();
        }
        self.history.push_back(record);
    }

    pub fn push_completed(&mut self, op: &FileOperation) {
        if let Some(record) = UndoRecord::from_completed(op) {
            self.push(record);
        }
    }

    pub fn undo_last(&mut self, sender: &Dispatcher) -> bool {
        if let Some(record) = self.history.pop_back() {
            record.execute_undo(sender);
            true
        } else {
            false
        }
    }

    pub fn undo_at(&mut self, index: usize, sender: &Dispatcher) -> bool {
        if let Some(record) = self.history.remove(index) {
            record.execute_undo(sender);
            true
        } else {
            false
        }
    }
}
