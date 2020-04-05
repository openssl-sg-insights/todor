use restson::{RestPath, Error, RestClient};

const URL_BASE: &str = "https://api.todoist.com/";

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Project {
    comment_count: usize,
    id: usize,
    name: String,
    color: usize,
    shared: bool
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Task {
    pub id: usize,
    pub project_id: usize,
    pub section_id: usize,
    pub content: String,
    pub completed: bool,
    pub label_ids: Vec<usize>,
    pub parent: Option<usize>,
    pub order: Option<usize>,
    pub priority: usize,
    pub due: Option<TodoistDate>,
    pub url: String
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct TodoistDate {
    pub string: String,
    pub date: String,  // Date in format YYYY-MM-DD corrected to user’s timezone.
    pub datetime: Option<String>, // date and time in RFC3339 format in UTC.
    pub timezone: Option<String>, // Only returned if exact due time set, user’s timezone
                                  // definition either in tzdata-compatible format (“Europe/Berlin”)
                                  // or as a string specifying east of UTC offset as “UTC±HH:MM”
                                  // (i.e. “UTC-01:00”).
}

pub trait TodoistClient {
    fn projects(&mut self) -> Vec<Project>;
    fn tasks(&mut self, project: &str) -> Vec<Task>;
}

pub struct TodoistRestClient {
    token: String,
    projects: Vec<Project>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
struct Projects ( pub Vec<Project> ); // alias to help deserialization

#[derive(Serialize,Deserialize,Debug,Clone)]
struct Tasks ( pub Vec<Task> ); // alias to help deserialization

// get("https://api.todoist.com/rest/v1/projects", headers={"Authorization": "Bearer %s" % your_token}).json()

impl RestPath<()> for Projects {
    fn get_path(_: ()) -> Result<String,Error> { Ok(String::from("rest/v1/projects")) }
}

impl RestPath<()> for Tasks {
    fn get_path(_: ()) -> Result<String,Error> { Ok(format!("rest/v1/tasks")) }
}

impl TodoistRestClient {
    pub fn new(token: String) -> TodoistRestClient {
        TodoistRestClient { token, projects: Vec::new() }
    }

    fn get_client(&mut self) -> RestClient {
        let mut client = RestClient::new(URL_BASE).unwrap();
        client.set_header("Authorization", format!("Bearer {}", self.token).as_str());
        client
    }
}
impl TodoistClient for TodoistRestClient {
    fn projects(&mut self) -> Vec<Project> {
        if self.projects.is_empty() {
            let mut client = self.get_client();
            self.projects = match client.get::<_, Projects>(()) {
                Ok(projects) => projects.0,
                Err(_) => Vec::new()
            };
        }

        self.projects.clone()
    }

    fn tasks(&mut self, project: &str) -> Vec<Task> {
        let mut client = self.get_client();
        let projects = self.projects();
        let selected_project = projects.iter().find(|p| p.name == project).
            expect(format!("No project named {}", project).as_str());

        // TODO: Somehow, I should be able to use `get_with` to pass the project id into this call.
        let tasks: Vec<Task> = client.get_with::<_, Tasks>((), &[("project_id", format!("{}", selected_project.id).as_str())]).unwrap().0.iter().
            filter(|t| t.project_id == selected_project.id).
            map(|t| t.to_owned()).
            collect();
        tasks
    }


}

