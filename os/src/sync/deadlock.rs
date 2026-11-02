use alloc::{vec, vec::Vec};

#[derive(Default)]
/// Available instances, current holdings and outstanding requests.
pub struct ResourceState {
    available: Vec<usize>,
    allocation: Vec<Vec<usize>>,
    need: Vec<Vec<usize>>,
}
impl ResourceState {
    /// Append a resource class, expanding every existing thread row.
    pub fn add_resource(&mut self, count: usize) {
        self.available.push(count);
        for row in &mut self.allocation {
            row.push(0);
        }
        for row in &mut self.need {
            row.push(0);
        }
    }
    fn ensure_thread(&mut self, tid: usize) {
        while self.allocation.len() <= tid {
            self.allocation.push(vec![0; self.available.len()]);
            self.need.push(vec![0; self.available.len()]);
        }
    }
    fn safe(&self) -> bool {
        let mut work = self.available.clone();
        let mut finish = vec![false; self.allocation.len()];
        loop {
            let mut progress = false;
            for i in 0..finish.len() {
                if !finish[i] && self.need[i].iter().zip(&work).all(|(n, w)| n <= w) {
                    for (w, a) in work.iter_mut().zip(&self.allocation[i]) {
                        *w += a;
                    }
                    finish[i] = true;
                    progress = true;
                }
            }
            if !progress {
                return finish.iter().all(|f| *f);
            }
        }
    }
    /// Tentatively add the request, rolling it back if it would make the
    /// pending-request graph unsafe. Never retain a rejected request.
    pub fn request(&mut self, tid: usize, id: usize, detect: bool) -> bool {
        self.ensure_thread(tid);
        self.need[tid][id] += 1;
        if detect && !self.safe() {
            self.need[tid][id] -= 1;
            false
        } else {
            true
        }
    }
    /// Called only after the primitive has actually acquired the resource,
    /// including after a blocked task wakes. Pending requests are not holdings.
    pub fn grant(&mut self, tid: usize, id: usize) {
        assert!(self.available[id] > 0 && self.need[tid][id] > 0);
        self.available[id] -= 1;
        self.need[tid][id] -= 1;
        self.allocation[tid][id] += 1;
    }
    /// Return one instance, accounting for producer-only semaphore up.
    pub fn release(&mut self, tid: usize, id: usize) {
        self.ensure_thread(tid);
        if self.allocation[tid][id] > 0 {
            self.allocation[tid][id] -= 1;
        }
        // Semaphore up may be performed by a producer that never did down.
        self.available[id] += 1;
    }
}

#[derive(Default)]
/// Independent resource graphs and the detection policy of one process.
pub struct DeadlockDetector {
    /// Whether unsafe resource requests must be rejected.
    pub enabled: bool,
    /// One-instance resources, one column per mutex.
    pub mutexes: ResourceState,
    /// Multi-instance resources, one column per semaphore.
    pub semaphores: ResourceState,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn self_deadlock_and_rejection_rollback() {
        let mut s = ResourceState::default();
        s.add_resource(1);
        assert!(s.request(0, 0, true));
        s.grant(0, 0);
        assert!(!s.request(0, 0, true));
        s.release(0, 0);
        assert!(s.request(0, 0, true));
        s.grant(0, 0);
    }
    #[test]
    fn circular_wait_and_safe_blocking() {
        let mut s = ResourceState::default();
        s.add_resource(1);
        s.add_resource(1);
        assert!(s.request(0, 0, true));
        s.grant(0, 0);
        assert!(s.request(1, 1, true));
        s.grant(1, 1);
        assert!(s.request(0, 1, true));
        assert!(!s.request(1, 0, true));
        s.release(1, 1);
        s.grant(0, 1);
    }
    #[test]
    fn multiple_instances_and_dynamic_rows() {
        let mut s = ResourceState::default();
        s.add_resource(2);
        assert!(s.request(0, 0, true));
        s.grant(0, 0);
        assert!(s.request(3, 0, true));
        s.grant(3, 0);
        assert!(s.request(0, 0, true));
        assert!(!s.request(3, 0, true));
        s.add_resource(1);
        assert!(s.request(8, 1, true));
        s.grant(8, 1);
    }
    #[test]
    fn disabled_detection_still_tracks_allocations() {
        let mut s = ResourceState::default();
        s.add_resource(1);
        assert!(s.request(0, 0, false));
        s.grant(0, 0);
        assert!(!s.request(0, 0, true));
    }

    #[test]
    fn resource_classes_and_processes_are_independent() {
        let mut first = DeadlockDetector::default();
        let mut second = DeadlockDetector::default();
        first.enabled = true;
        for detector in [&mut first, &mut second] {
            detector.mutexes.add_resource(1);
            detector.semaphores.add_resource(1);
            assert!(detector.mutexes.request(0, 0, detector.enabled));
            detector.mutexes.grant(0, 0);
            assert!(detector.semaphores.request(0, 0, detector.enabled));
            detector.semaphores.grant(0, 0);
        }
        assert!(!first.mutexes.request(0, 0, first.enabled));
        second.enabled = true;
        assert!(!second.semaphores.request(0, 0, second.enabled));
    }
}
