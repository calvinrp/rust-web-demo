use wstd::http::request::JsonRequest;
use wstd::io::{copy, AsyncWrite};
use wstd::{
    self,
    http::{
        body::{BodyForthcoming, IncomingBody},
        request::Builder,
        server::{Finished, Responder},
        Client, IntoBody, Request, Response, StatusCode,
    },
};

wstd::http_server! {
    async fn main(request: Request<IncomingBody>, responder: Responder) -> Finished {
        // read github API token from env var
        let token = match std::env::var("TOKEN") {
            Ok(token) => token,
            Err(_) => {
                let response = Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body("Missing `TOKEN` environment variable".into_body())
                    .unwrap();
                return responder.respond(response).await;
            }
        };

        match request.uri().path() {
            "/gh/repo" => {
                http_gh_repo(request, responder, &token).await
            },
            "/gh/issue" => {
                http_gh_issue(request, responder, &token).await
            }
            "/gh/close_issue" =>{
                http_gh_close(request, responder, &token).await
            }
            "/gh/create" => {
                http_gh_create(request, responder, &token).await
            },
            "/gh/repos" => {
                http_gh_repos(request, responder, &token).await
            },
            "/gh/user" => {
                http_gh_user(request, responder, &token).await
            },
            "/gh/create_comment" => {
                http_gh_create_comment(request, responder, &token).await
            }
            "/gh/comments" => {
                http_gh_comments(request, responder, &token).await
            }
            "/issue" => {
                http_issue(request, responder).await
            },
            "/create" => {
                http_create(request, responder).await
            },
            "/repos" => {
                http_repos(request, responder).await
            }
            "/repo" => {
                http_repo(request, responder).await
            }
            _ => {
                http_repos(request, responder).await
            },
        }
    }
}

const ISSUE: &[u8] = b"
<html>
<head>
<script src=\"https://cdn.tailwindcss.com\">
</script>
<script>
    tailwind.config = {
    theme: {
        extend: {},
        fontFamily: {
        sans: [\"Inter var\", \"sans-serif\"],
        mono: [\"Roboto Mono\", \"monospace\"],
        },
    },
    }
</script>
<script src=\"https://cdnjs.cloudflare.com/ajax/libs/qs/6.14.0/qs.min.js\"></script>
<script src=\"https://cdn.jsdelivr.net/npm/marked/marked.min.js\"></script>
<title></title>
    </head>
    <body class=\"bg-slate-900 text-white p-5\">
<h1 class=\"text-3xl text-slate-100 font-black mb-10\" id=\"issue-title\"></h1>
<div class=\"text-slate-300 bg-slate-800 rounded p-3 inline-block\" id=\"issue-body\">
</div>
<div class=\"mt-10\">
<a href=\"/\" class=\"underline decoration-sky-500 decoration-solid decoration-2 text-slate-200 hover:text-white\">Back to issue list</a>
</div>
<script type=\"module\">
    const search = window.location.search;
    const query = Qs.parse(search);
    const urlParams = new URLSearchParams(window.location.search);
    async function getIssue() {
        let res = await fetch(`/gh/issue?${urlParams.toString()}`);
        return res;
    }
    const issuesRes = await getIssue();
    let issue = await issuesRes.json();
    
    async function getComments() {
        return await fetch(`/gh/comments?${urlParams.toString()}`)
    }
    const h1 = document.getElementById('issue-title');
    h1.textContent = issue.title;
    document.title = issue.title;
    if (issue.user.login === query[`?owner`]) {
        let button = document.createElement(`Button`);
        let text = document.createTextNode(`Close Issue`);
        button.classList.add(\"text-slate-300\", \"bg-slate-800\", \"rounded\", \"p-3\", \"inline-block\", \"hover:text-white\", \"hover:bg-slate-700\")
        button.appendChild(text);
        async function clickHandler() {
            await fetch('/gh/close_issue', {
                method: \"PATCH\",
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    owner: query[`?owner`],
                    repo: query.repo,
                    number: Number(query.number),
                    state: \"closed\"
                })
            })
        }
        button.addEventListener('click', clickHandler);
        document.body.appendChild(button);
    }

    const issueBody = document.getElementById('issue-body');
    issueBody.setHTMLUnsafe(issue.body);
    const commentsRes = await getComments();
    const comments = await commentsRes.json();
    let ul = document.createElement(\"ul\");
    ul.classList.add('mt-5', 'list-disc', 'pl-5');
    for (const comment of comments) {
        let li = document.createElement(\"li\");
        const createdDiv = document.createElement(\"div\");
        const createdText = document.createTextNode(comment.created_at);
        createdDiv.appendChild(createdText);
        const authorDiv = document.createElement(\"div\");
        const authorText = document.createTextNode(comment.user.login);
        authorDiv.appendChild(authorText)
        let text;
        // if (comment.user.login === query[`?owner`]) {
        //     text = document.createElement(\"textarea\");
        //     text.innerText = comment.body;
        //     text.cols = \"50\";
        //     text.rows = \"4\";
        //     text.classList.add(\"bg-slate-700\", \"text-slate-100\", \"py-2\", \"px-3\", \"rounded\");
        //     const button = document.createElement(\"button\")
        //     button.innerText = `Edit Comment`
        //     async function clickHandler() {
        //         const body = JSON.stringify({
        //             owner: query[`?owner`],
        //             repo: query.repo,
        //             number: Number(query.number),
        //             body: `<pre>${text.innerText}</pre>`,
        //         });
        //         let res = await fetch(`/gh/create_comment`, {
        //             method: \"POST\",
        //             headers: { 'Content-Type': 'application/json' },
        //             body,
        //         });
        //     }
        //     button.addEventListener(\"click\", clickHandler);
        //     document.body.appendChild(button);
        // } else {
            text = document.createElement(\"div\");
            text.innerHTML = marked.parse(comment.body);
            text.classList.add(\"text-slate-300\", \"bg-slate-800\", \"rounded\", \"p-3\", \"inline-block\");
        // }

        li.appendChild(createdDiv)
        li.appendChild(authorDiv);
        // const pre = document.createElement(\"pre\");
        // pre.appendChild(text);
        // li.appendChild(pre);
        li.appendChild(text);
        ul.appendChild(li);
    }
    document.body.appendChild(ul);
</script>
<div class=\"mb-10\">
<div>Create a comment</div>
    <form id=\"form\">
        <div class=\"mt-1\">
        <textarea id=\"issue-body\" name=\"body\", rows=\"4\" cols=\"50\" class=\"bg-slate-700 text-slate-100 py-2 px-3 rounded\">Write a comment here</textarea>
        </div>
        <div class=\"mt-5\">
        <input type=\"submit\" value=\"Submit\" class=\"cursor-pointer inline-block py-2 px-3 bg-slate-800 rounded-lg text-slate-200 hover:text-white hover:bg-slate-700\" />
        </div>
    </form>
</div>
<script type=\"module\">
    const search = window.location.search;
    const query = Qs.parse(search);
    const form = document.getElementById(\"form\");
    async function getUser() {
        return await fetch(`/gh/user`)
    }
    const userRes = await getUser();
    const owner = await userRes.json();
    async function submitHandler(e) {
        const data = new FormData(e.target);
        e.preventDefault()
        const body = JSON.stringify({
            owner: owner.login,
            repo: query[`repo`],
            number: Number(query.number),
            body: data.get('body'),
        });
        let res = await fetch(`/gh/create_comment`, {
            method: \"POST\",
            headers: { 'Content-Type': 'application/json' },
            body,
        });
        let resBody = await res.json();

    }
    form.addEventListener(\"submit\", submitHandler);
</script>
</body>
</html>";

const CREATE: &[u8] = b"
<html>
<head>
<title>Create new issue</title>
<script src=\"https://cdn.tailwindcss.com\">
</script>
<script>
  tailwind.config = {
    theme: {
      extend: {},
      fontFamily: {
        sans: [\"Inter var\", \"sans-serif\"],
        mono: [\"Roboto Mono\", \"monospace\"],
      },
  },
  }
</script>
<script src=\"https://cdnjs.cloudflare.com/ajax/libs/qs/6.14.0/qs.min.js\"></script>
</head>
    <body class=\"bg-slate-900 text-white p-5\">
    <script type=\"module\">
      const search = window.location.search;
      const query = Qs.parse(search);
      const form = document.getElementById(\"form\");
      async function getUser() {
        return await fetch(`/gh/user`)
      }
      const userRes = await getUser();
      const owner = await userRes.json();
      async function submitHandler(e) {
        const data = new FormData(e.target);
        e.preventDefault()
        const body = JSON.stringify({
          owner: owner.login,
          repo: query[`?repo`],
          title: data.get('title'),
          body: data.get('body'),
        });
        let res = await fetch(`/gh/create`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body,
        });
        let resBody = await res.json();

        window.location.pathname = '/';
      }
      form.addEventListener(\"submit\", submitHandler);
    </script>
<h1 class=\"text-3xl text-slate-100 font-black mb-10\">Create new issue</h1>
<div class=\"mb-10\">
<form id=\"form\">
  <label for=\"title\">Issue Title</label>
  <div class=\"mb-3 mt-1\">
      <input type=\"text\" id=\"title\" name=\"title\" class=\"bg-slate-700 text-slate-100 py-2 px-3 rounded\">
  </div>
  <label for=\"body\">Issue Body</label>
  <div class=\"mt-1\">
  <textarea id=\"issue-body\" name=\"body\", rows=\"4\" cols=\"50\" class=\"bg-slate-700 text-slate-100 py-2 px-3 rounded\">Write your issue here</textarea>
  </div>
  <div class=\"mt-5\">
   <input type=\"submit\" value=\"Submit\" class=\"cursor-pointer inline-block py-2 px-3 bg-slate-800 rounded-lg text-slate-200 hover:text-white hover:bg-slate-700\" />
  </div>
</form>
</div>
<a href=\"/\" class=\"underline decoration-sky-500 decoration-solid decoration-2 text-slate-200 hover:text-white\">Back to issue list</a>
</body>
</html>";

const REPOS: &[u8] = b"
<html>
<head>
<script src=\"https://cdn.tailwindcss.com\">
</script>
<script>
    tailwind.config = {
    theme: {
        extend: {},
        fontFamily: {
        sans: [\"Inter var\", \"sans-serif\"],
        mono: [\"Roboto Mono\", \"monospace\"],
        },
    },
    }
</script>
<title></title>
    </head>
    <body class=\"bg-slate-900 text-white p-5\">
<h1 class=\"text-3xl text-slate-100 font-black mb-10\" id=\"repos\"></h1>
<div class=\"mt-10\">
    <a href=\"/\" class=\"underline decoration-sky-500 decoration-solid decoration-2 text-slate-200 hover:text-white\">Back to issue list</a>
</div>
<script type=\"module\">
    async function getRepos(repo) {
        return await fetch(`/gh/repos?repo=${repo}`)
    }
    const reposRes = await getRepos(\"macovedj\");
    let repos = await reposRes.json();
    
    const h1 = document.getElementById('repos');
    h1.textContent = `Your Repositories`;
    document.title = `Your Repositories`;
    let ul = document.createElement(\"ul\");
    ul.classList.add('mt-5', 'list-disc', 'pl-5');
    for (const repo of repos) {
        let li = document.createElement(\"li\");
        let a = document.createElement(\"a\");
        a.classList.add(\"underline\", \"decoration-sky-500\", \"decoration-solid\", \"decoration-2\", 'text-slate-200', 'hover:text-white');
        li.appendChild(a);
        const linkText = document.createTextNode(repo.name);
        a.appendChild(linkText);
        a.href = `/repo?repo=${repo.name}`;
        ul.appendChild(li);
    }
    document.body.appendChild(ul);

</script>
</body>
</html>";

const REPO: &[u8] = b"
<html>
<head>
<script src=\"https://cdn.tailwindcss.com\">
</script>
<script>
    tailwind.config = {
    theme: {
        extend: {},
        fontFamily: {
        sans: [\"Inter var\", \"sans-serif\"],
        mono: [\"Roboto Mono\", \"monospace\"],
        },
    },
    }
</script>
<title></title>
</head>
<body class=\"bg-slate-900 text-white p-5\">
<h1 class=\"text-3xl text-slate-100 font-black mb-10\" id=\"open-issues\">Open</h1>
<div class=\"mt-10\">
    <a href=\"/\" class=\"underline decoration-sky-500 decoration-solid decoration-2 text-slate-200 hover:text-white\">Back to issue list</a>
</div>
<script src=\"https://cdnjs.cloudflare.com/ajax/libs/qs/6.14.0/qs.min.js\"></script>
<script type=\"module\">
    const search = window.location.search;
    const query = Qs.parse(search);

    async function getIssues(repo) {
        return await fetch(`/gh/repo?repo=${repo}`)
    }
    async function getUser() {
        return await fetch(`/gh/user`)
    }
    const resp = await getIssues(query[`?repo`]);
    let issues = await resp.json();
    const open = issues.filter(issue => issue.state === 'open');
    const closed = issues.filter(issue => issue.state === 'closed');
    
    const h1 = document.getElementById('open-issues');
    h1.textContent = `${query['?repo']} issues`;
    document.title = query[`?repo`];
    const userRes = await getUser();
    const user = await userRes.json();
    
    let openIssueHeader = document.createElement(\"h1\");
    openIssueHeader.textContent = `Open Issues`;
    document.body.appendChild(openIssueHeader);
    let ul = document.createElement(\"ul\");
    ul.classList.add('mt-5', 'list-disc', 'pl-5');
    for (const issue of open) {
        let li = document.createElement(\"li\");
        let a = document.createElement(\"a\");
        a.classList.add(\"underline\", \"decoration-sky-500\", \"decoration-solid\", \"decoration-2\", 'text-slate-200', 'hover:text-white');
        li.appendChild(a);
        const linkText = document.createTextNode(issue.title);
        a.appendChild(linkText);
        a.href = `/issue?owner=${user.login}&repo=${query[`?repo`]}&number=${issue.number}`;
        ul.appendChild(li);
    }
    document.body.appendChild(ul);
    let closedIssueHeader = document.createElement(\"h2\");
    let closedUl = document.createElement(\"ul\");
    closedUl.classList.add('mt-5', 'list-disc', 'pl-5');
    closedIssueHeader.textContent = `Closed Issues`;
    document.body.appendChild(closedIssueHeader)
    for (const issue of closed) {
        let li = document.createElement(\"li\");
        let a = document.createElement(\"a\");
        a.classList.add(\"underline\", \"decoration-sky-500\", \"decoration-solid\", \"decoration-2\", 'text-slate-200', 'hover:text-white');
        li.appendChild(a);
        const linkText = document.createTextNode(issue.title);
        a.appendChild(linkText);
        a.href = `/issue?owner=${user.login}&repo=${query[`?repo`]}&number=${issue.number}`;
        closedUl.appendChild(li);
    }
    document.body.appendChild(closedUl);
    let a = document.createElement(\"a\")
    a.classList.add(\"underline\", \"decoration-sky-500\", \"decoration-solid\", \"decoration-2\", 'text-slate-200', 'hover:text-white');
    const linkText = document.createTextNode(`Create an issue`);
    a.appendChild(linkText);
    a.href = `/create?repo=${query[`?repo`]}`;
    document.body.appendChild(a);
    </script>
    </div>
    </body>
    </html>";

#[derive(Debug, PartialEq)]
struct Issue {
    title: String,
}

#[derive(Debug, PartialEq)]
struct User {
    login: String,
}

#[derive(Debug, PartialEq)]
struct IssueResponse {
    title: String,
    body: String,
}

#[derive(Debug, PartialEq)]
struct ReqBody {
    owner: String,
    repo: String,
    title: String,
    body: String,
}

#[derive(Debug, PartialEq)]
struct CommentBody {
    owner: String,
    repo: String,
    body: String,
    number: u8,
}

#[derive(Debug, PartialEq)]
struct Comment {
    body: String,
}
async fn http_issue(_request: Request<IncomingBody>, responder: Responder) -> Finished {
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(ISSUE).await;
    Finished::finish(body, result, None)
}

async fn http_create(_request: Request<IncomingBody>, responder: Responder) -> Finished {
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(CREATE).await;
    Finished::finish(body, result, None)
}

#[derive(Debug, PartialEq)]
struct IssueParams {
    owner: String,
    repo: String,
    number: u8,
}
async fn http_gh_issue(
    request: Request<IncomingBody>,
    responder: Responder,
    token: &str,
) -> Finished {
    let builder = Builder::new();
    let query = request.uri().path_and_query().unwrap().query().unwrap();

    let mut pieces = query.split("=");
    pieces.next().unwrap();
    let owner = pieces.next().unwrap().split("&").next().unwrap();
    let repo = pieces.next().unwrap().split("&").next().unwrap();
    let number = pieces.next().unwrap().split("&").next().unwrap();
    let request = builder
        .uri(format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            owner, repo, number
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", token)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .body(wstd::io::empty())
        .unwrap();

    let client = Client::new();
    let mut resp = client.send(request).await.unwrap();

    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = copy(resp.body_mut(), &mut body).await;
    Finished::finish(body, result, None)
}

#[derive(Debug, PartialEq)]
struct CloseRequest {
    owner: String,
    repo: String,
    number: u8,
    state: String,
}
async fn http_gh_create(
    request: Request<IncomingBody>,
    responder: Responder,
    token: &str,
) -> Finished {
    let mut issue: ReqBody = request
        .into_body()
        .json()
        .await
        .expect("failed to deserialize");
    let md_opt = markdown::Options::gfm();
    let issue_body = markdown::to_html_with_options(&issue.body, &md_opt).unwrap();
    issue.body = issue_body;
    let builder = Builder::new();
    let request = builder
        .uri(format!(
            "https://api.github.com/repos/{}/{}/issues",
            issue.owner, issue.repo
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", token)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .method("POST")
        .json(&issue)
        .unwrap();

    let client = Client::new();
    let mut resp = client.send(request).await.unwrap();

    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = copy(resp.body_mut(), &mut body).await;
    Finished::finish(body, result, None)
}
async fn http_gh_close(
    request: Request<IncomingBody>,
    responder: Responder,
    token: &str,
) -> Finished {
    let issue: CloseRequest = request.into_body().json().await.unwrap();
    let builder = Builder::new();
    let request = builder
        .uri(format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            issue.owner, issue.repo, issue.number
        ))
        .header("Authorization", token)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .method("POST")
        .json(&issue)
        .unwrap();
    let client = Client::new();
    let resp = client.send(request).await.unwrap();
    let bytes = resp.into_body().bytes().await.unwrap();
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(&bytes).await;

    Finished::finish(body, result, None)
}

async fn http_repos(_request: Request<IncomingBody>, responder: Responder) -> Finished {
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(REPOS).await;
    Finished::finish(body, result, None)
}

#[derive(Debug, PartialEq)]
struct Repo {
    name: String,
}
async fn http_gh_repos(
    _request: Request<IncomingBody>,
    responder: Responder,
    token: &str,
) -> Finished {
    let builder = Builder::new();
    let request = builder
        .uri("https://api.github.com/users/macovedj/repos")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", token)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .body(wstd::io::empty())
        .unwrap();
    let client = Client::new();
    let resp = client.send(request).await.unwrap();
    let json: Vec<Repo> = resp.into_body().json().await.unwrap();
    let bytes = serde_json::to_vec(&json).unwrap();
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(&bytes).await;
    Finished::finish(body, result, None)
}

async fn http_repo(_request: Request<IncomingBody>, responder: Responder) -> Finished {
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(REPO).await;
    Finished::finish(body, result, None)
}

#[derive(Debug, PartialEq)]
struct Params {
    repo: String,
}
async fn http_gh_repo(
    request: Request<IncomingBody>,
    responder: Responder,
    token: &str,
) -> Finished {
    let user_builder = Builder::new();
    let user_request = user_builder
        .uri("https://api.github.com/user")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", token)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .body(wstd::io::empty())
        .unwrap();
    let client = Client::new();
    let user: User = client
        .send(user_request)
        .await
        .unwrap()
        .into_body()
        .json()
        .await
        .unwrap();

    let query = request.uri().path_and_query().unwrap().query().unwrap();
    let repo = query.split("=").last().unwrap();

    let builder = Builder::new();
    let client = Client::new();
    let request = builder
        .uri(format!(
            "https://api.github.com/repos/{}/{}/issues?state=all",
            user.login, repo
        ))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", token)
        .body(wstd::io::empty())
        .unwrap();
    let resp = client.send(request).await.unwrap();
    let bytes = resp.into_body().bytes().await.unwrap();
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(&bytes).await;
    Finished::finish(body, result, None)
}

async fn http_gh_user(
    _request: Request<IncomingBody>,
    responder: Responder,
    token: &str,
) -> Finished {
    let builder = Builder::new();
    let request = builder
        .uri("https://api.github.com/user")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", token)
        .body(wstd::io::empty())
        .unwrap();
    let client = Client::new();
    let resp = client.send(request).await.unwrap();
    let bytes = resp.into_body().bytes().await.unwrap();
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(&bytes).await;
    Finished::finish(body, result, None)
}

async fn http_gh_comments(
    request: Request<IncomingBody>,
    responder: Responder,
    token: &str,
) -> Finished {
    let builder = Builder::new();
    let query = request.uri().path_and_query().unwrap().query().unwrap();

    let mut pieces = query.split("=");
    pieces.next().unwrap();
    let owner = pieces.next().unwrap().split("&").next().unwrap();
    let repo = pieces.next().unwrap().split("&").next().unwrap();
    let number = pieces.next().unwrap().split("&").next().unwrap();
    let request = builder
        .uri(format!(
            "https://api.github.com/repos/{}/{}/issues/{}/comments",
            owner, repo, number
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", token)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .body(wstd::io::empty())
        .unwrap();

    let client = Client::new();
    let resp = client.send(request).await.unwrap();
    let bytes = resp.into_body().bytes().await.unwrap();
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(&bytes).await;

    Finished::finish(body, result, None)
}

async fn http_gh_create_comment(
    request: Request<IncomingBody>,
    responder: Responder,
    token: &str,
) -> Finished {
    let issue: CommentBody = request.into_body().json().await.unwrap();
    let builder = Builder::new();
    let comment = Comment {
        // body: format!("{},\n\n -Developer Danny", issue.body)
        body: issue.body,
    };
    let request = builder
        .uri(format!(
            "https://api.github.com/repos/{}/{}/issues/{}/comments",
            issue.owner, issue.repo, issue.number
        ))
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", token)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .method("POST")
        .json(&comment)
        .unwrap();
    let client = Client::new();
    let resp = client.send(request).await.unwrap();
    let bytes = resp.into_body().bytes().await.unwrap();
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body.write_all(&bytes).await;

    Finished::finish(body, result, None)
}

// SERDE macro expansion:

#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Issue {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state =
                _serde::Serializer::serialize_struct(__serializer, "Issue", false as usize + 1)?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "title",
                &self.title,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Issue {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "title" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"title" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Issue>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Issue;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct Issue")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct Issue with 1 element",
                            ));
                        }
                    };
                    _serde::__private::Ok(Issue { title: __field0 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("title"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("title")?,
                    };
                    _serde::__private::Ok(Issue { title: __field0 })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["title"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Issue",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<Issue>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for User {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state =
                _serde::Serializer::serialize_struct(__serializer, "User", false as usize + 1)?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "login",
                &self.login,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for User {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "login" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"login" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<User>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = User;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct User")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct User with 1 element",
                            ));
                        }
                    };
                    _serde::__private::Ok(User { login: __field0 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("login"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("login")?,
                    };
                    _serde::__private::Ok(User { login: __field0 })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["login"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "User",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<User>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};

#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for IssueResponse {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "IssueResponse",
                false as usize + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "title",
                &self.title,
            )?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "body", &self.body)?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for IssueResponse {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "title" => _serde::__private::Ok(__Field::__field0),
                        "body" => _serde::__private::Ok(__Field::__field1),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"title" => _serde::__private::Ok(__Field::__field0),
                        b"body" => _serde::__private::Ok(__Field::__field1),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<IssueResponse>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = IssueResponse;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct IssueResponse")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct IssueResponse with 2 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct IssueResponse with 2 elements",
                            ));
                        }
                    };
                    _serde::__private::Ok(IssueResponse {
                        title: __field0,
                        body: __field1,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("title"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("body"),
                                    );
                                }
                                __field1 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("title")?,
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => _serde::__private::de::missing_field("body")?,
                    };
                    _serde::__private::Ok(IssueResponse {
                        title: __field0,
                        body: __field1,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["title", "body"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "IssueResponse",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<IssueResponse>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};

#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for ReqBody {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "ReqBody",
                false as usize + 1 + 1 + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "owner",
                &self.owner,
            )?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "repo", &self.repo)?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "title",
                &self.title,
            )?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "body", &self.body)?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for ReqBody {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        3u64 => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "owner" => _serde::__private::Ok(__Field::__field0),
                        "repo" => _serde::__private::Ok(__Field::__field1),
                        "title" => _serde::__private::Ok(__Field::__field2),
                        "body" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"owner" => _serde::__private::Ok(__Field::__field0),
                        b"repo" => _serde::__private::Ok(__Field::__field1),
                        b"title" => _serde::__private::Ok(__Field::__field2),
                        b"body" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<ReqBody>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = ReqBody;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct ReqBody")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct ReqBody with 4 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct ReqBody with 4 elements",
                            ));
                        }
                    };
                    let __field2 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"struct ReqBody with 4 elements",
                            ));
                        }
                    };
                    let __field3 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                3usize,
                                &"struct ReqBody with 4 elements",
                            ));
                        }
                    };
                    _serde::__private::Ok(ReqBody {
                        owner: __field0,
                        repo: __field1,
                        title: __field2,
                        body: __field3,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field2: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field3: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("owner"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("repo"),
                                    );
                                }
                                __field1 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("title"),
                                    );
                                }
                                __field2 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field3 => {
                                if _serde::__private::Option::is_some(&__field3) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("body"),
                                    );
                                }
                                __field3 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("owner")?,
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => _serde::__private::de::missing_field("repo")?,
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => _serde::__private::de::missing_field("title")?,
                    };
                    let __field3 = match __field3 {
                        _serde::__private::Some(__field3) => __field3,
                        _serde::__private::None => _serde::__private::de::missing_field("body")?,
                    };
                    _serde::__private::Ok(ReqBody {
                        owner: __field0,
                        repo: __field1,
                        title: __field2,
                        body: __field3,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["owner", "repo", "title", "body"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "ReqBody",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<ReqBody>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};

#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for CommentBody {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "CommentBody",
                false as usize + 1 + 1 + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "owner",
                &self.owner,
            )?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "repo", &self.repo)?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "body", &self.body)?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "number",
                &self.number,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for CommentBody {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        3u64 => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "owner" => _serde::__private::Ok(__Field::__field0),
                        "repo" => _serde::__private::Ok(__Field::__field1),
                        "body" => _serde::__private::Ok(__Field::__field2),
                        "number" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"owner" => _serde::__private::Ok(__Field::__field0),
                        b"repo" => _serde::__private::Ok(__Field::__field1),
                        b"body" => _serde::__private::Ok(__Field::__field2),
                        b"number" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<CommentBody>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = CommentBody;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct CommentBody")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct CommentBody with 4 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct CommentBody with 4 elements",
                            ));
                        }
                    };
                    let __field2 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"struct CommentBody with 4 elements",
                            ));
                        }
                    };
                    let __field3 = match _serde::de::SeqAccess::next_element::<u8>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                3usize,
                                &"struct CommentBody with 4 elements",
                            ));
                        }
                    };
                    _serde::__private::Ok(CommentBody {
                        owner: __field0,
                        repo: __field1,
                        body: __field2,
                        number: __field3,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field2: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field3: _serde::__private::Option<u8> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("owner"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("repo"),
                                    );
                                }
                                __field1 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("body"),
                                    );
                                }
                                __field2 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field3 => {
                                if _serde::__private::Option::is_some(&__field3) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "number",
                                        ),
                                    );
                                }
                                __field3 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<u8>(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("owner")?,
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => _serde::__private::de::missing_field("repo")?,
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => _serde::__private::de::missing_field("body")?,
                    };
                    let __field3 = match __field3 {
                        _serde::__private::Some(__field3) => __field3,
                        _serde::__private::None => _serde::__private::de::missing_field("number")?,
                    };
                    _serde::__private::Ok(CommentBody {
                        owner: __field0,
                        repo: __field1,
                        body: __field2,
                        number: __field3,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["owner", "repo", "body", "number"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "CommentBody",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<CommentBody>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};

#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Comment {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state =
                _serde::Serializer::serialize_struct(__serializer, "Comment", false as usize + 1)?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "body", &self.body)?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Comment {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "body" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"body" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Comment>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Comment;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct Comment")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct Comment with 1 element",
                            ));
                        }
                    };
                    _serde::__private::Ok(Comment { body: __field0 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("body"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("body")?,
                    };
                    _serde::__private::Ok(Comment { body: __field0 })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["body"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Comment",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<Comment>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};

#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for IssueParams {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "IssueParams",
                false as usize + 1 + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "owner",
                &self.owner,
            )?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "repo", &self.repo)?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "number",
                &self.number,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for IssueParams {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "owner" => _serde::__private::Ok(__Field::__field0),
                        "repo" => _serde::__private::Ok(__Field::__field1),
                        "number" => _serde::__private::Ok(__Field::__field2),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"owner" => _serde::__private::Ok(__Field::__field0),
                        b"repo" => _serde::__private::Ok(__Field::__field1),
                        b"number" => _serde::__private::Ok(__Field::__field2),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<IssueParams>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = IssueParams;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct IssueParams")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct IssueParams with 3 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct IssueParams with 3 elements",
                            ));
                        }
                    };
                    let __field2 = match _serde::de::SeqAccess::next_element::<u8>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"struct IssueParams with 3 elements",
                            ));
                        }
                    };
                    _serde::__private::Ok(IssueParams {
                        owner: __field0,
                        repo: __field1,
                        number: __field2,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field2: _serde::__private::Option<u8> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("owner"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("repo"),
                                    );
                                }
                                __field1 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "number",
                                        ),
                                    );
                                }
                                __field2 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<u8>(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("owner")?,
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => _serde::__private::de::missing_field("repo")?,
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => _serde::__private::de::missing_field("number")?,
                    };
                    _serde::__private::Ok(IssueParams {
                        owner: __field0,
                        repo: __field1,
                        number: __field2,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["owner", "repo", "number"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "IssueParams",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<IssueParams>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};

#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for CloseRequest {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "CloseRequest",
                false as usize + 1 + 1 + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "owner",
                &self.owner,
            )?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "repo", &self.repo)?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "number",
                &self.number,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "state",
                &self.state,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for CloseRequest {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        3u64 => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "owner" => _serde::__private::Ok(__Field::__field0),
                        "repo" => _serde::__private::Ok(__Field::__field1),
                        "number" => _serde::__private::Ok(__Field::__field2),
                        "state" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"owner" => _serde::__private::Ok(__Field::__field0),
                        b"repo" => _serde::__private::Ok(__Field::__field1),
                        b"number" => _serde::__private::Ok(__Field::__field2),
                        b"state" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<CloseRequest>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = CloseRequest;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct CloseRequest")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct CloseRequest with 4 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct CloseRequest with 4 elements",
                            ));
                        }
                    };
                    let __field2 = match _serde::de::SeqAccess::next_element::<u8>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"struct CloseRequest with 4 elements",
                            ));
                        }
                    };
                    let __field3 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                3usize,
                                &"struct CloseRequest with 4 elements",
                            ));
                        }
                    };
                    _serde::__private::Ok(CloseRequest {
                        owner: __field0,
                        repo: __field1,
                        number: __field2,
                        state: __field3,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field2: _serde::__private::Option<u8> = _serde::__private::None;
                    let mut __field3: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("owner"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("repo"),
                                    );
                                }
                                __field1 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "number",
                                        ),
                                    );
                                }
                                __field2 = _serde::__private::Some(
                                    _serde::de::MapAccess::next_value::<u8>(&mut __map)?,
                                );
                            }
                            __Field::__field3 => {
                                if _serde::__private::Option::is_some(&__field3) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("state"),
                                    );
                                }
                                __field3 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("owner")?,
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => _serde::__private::de::missing_field("repo")?,
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => _serde::__private::de::missing_field("number")?,
                    };
                    let __field3 = match __field3 {
                        _serde::__private::Some(__field3) => __field3,
                        _serde::__private::None => _serde::__private::de::missing_field("state")?,
                    };
                    _serde::__private::Ok(CloseRequest {
                        owner: __field0,
                        repo: __field1,
                        number: __field2,
                        state: __field3,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["owner", "repo", "number", "state"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "CloseRequest",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<CloseRequest>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};

#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Repo {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state =
                _serde::Serializer::serialize_struct(__serializer, "Repo", false as usize + 1)?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "name", &self.name)?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Repo {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "name" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"name" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Repo>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Repo;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct Repo")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct Repo with 1 element",
                            ));
                        }
                    };
                    _serde::__private::Ok(Repo { name: __field0 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("name"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("name")?,
                    };
                    _serde::__private::Ok(Repo { name: __field0 })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["name"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Repo",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<Repo>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};

#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Params {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state =
                _serde::Serializer::serialize_struct(__serializer, "Params", false as usize + 1)?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "repo", &self.repo)?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Params {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "repo" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"repo" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Params>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Params;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct Params")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct Params with 1 element",
                            ));
                        }
                    };
                    _serde::__private::Ok(Params { repo: __field0 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("repo"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("repo")?,
                    };
                    _serde::__private::Ok(Params { repo: __field0 })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["repo"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Params",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<Params>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
