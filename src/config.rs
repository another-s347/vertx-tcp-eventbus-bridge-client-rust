use rayon::{ThreadPool,ThreadPoolBuilder};

pub struct Configuration{
    pub thread_pool:Option<ThreadPool>
}

impl Configuration{
    pub fn new()->Configuration{
        Configuration{
            thread_pool:None
        }
    }

    pub fn thread_num(mut self, thread_num:usize) ->Configuration{
        let tp=ThreadPoolBuilder::new().num_threads(thread_num).build().unwrap();
        self.thread_pool =Some(tp);
        self
    }

    pub fn thread_pool(mut self,tp:ThreadPool)->Configuration{
        self.thread_pool =Some(tp);
        self
    }
}