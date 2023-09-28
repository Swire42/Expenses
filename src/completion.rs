#[derive(Debug, Clone)]
pub struct Completor {
    dict: Vec<String>,
    matches: Vec<String>,
}

impl Completor {
    pub fn new(dict: Vec<String>) -> Self {
        Self{matches: dict.clone(), dict}
    }

    pub fn update(&mut self, curr: &String) {
        let lower = curr.to_lowercase();
        self.matches = self.dict.iter().filter(|x| x.to_lowercase().contains(&lower)).map(Clone::clone).collect();
        self.matches.sort_by_cached_key(|x| x.to_lowercase().find(&lower).unwrap());
    }

    pub fn matches(&self) -> &Vec<String> {
        &self.matches
    }

    pub fn contains(&self, text: &String) -> bool {
        self.dict.contains(text)
    }
}
