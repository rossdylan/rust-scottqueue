# rust-scottqueue
## The queues of Michael and Scott implemented in rust


Currently Rust only has multi producer single consumer queues. I wanted to have
a multi producer, multi consumer queue. So I read the (Michael/Scott paper)[https://www.cs.rochester.edu/research/synchronization/pseudocode/queues.html] and attempted to implement them. Currently the
only stable queue is scottqueue::tlqueue which is the Two Lock queue described
in the paper. Work on the Non-Blocking Concurrent Queue is still in progress.
