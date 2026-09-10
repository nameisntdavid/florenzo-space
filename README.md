# Personal Blog

A server-rendered blog built with Rust, Axum, Sanity CMS, Redis, and HTMX. Deployed on Vercel via Docker.

## TODO
- redesign website
- have it use my github pfp as the main logo or something
- remove test artifacts
- add api rate limiting (please dont murder my website)

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | [Axum](https://github.com/tokio-rs/axum) — async Rust web framework |
| Templates | [Askama](https://github.com/askama-rs/askama) — compile-time Jinja2-style templates |
| Frontend | [HTMX](https://htmx.org/) — server-driven interactivity without a JS framework |
| CMS | [Sanity](https://www.sanity.io/) — headless content management with GROQ queries |
| Cache | [Redis](https://redis.io/) via [deadpool-redis](https://github.com/deadpool-rs/deadpool) — connection-pooled caching |
| Deployment | [Vercel](https://vercel.com/) — Docker-based serverless deployment |

## Features

- Server-side rendered pages with HTMX for partial updates
- Sanity CMS for content authoring (separate Studio project)
- Redis caching layer with configurable TTLs
- Emoji reaction system (no account required)
- Full-text search via GROQ text matching
- RSS/Atom feed at `/feed.xml`
- Dark mode support via `prefers-color-scheme`
- Responsive, minimal CSS (no framework)

## Project Structure

```
src/
├── main.rs              # Entry point, router, AppState
├── sanity/              # Sanity GROQ client, data models, queries
├── cache/               # Redis cache-through layer
├── portable_text/       # Sanity Portable Text → HTML renderer
└── routes/              # Axum route handlers
    ├── home.rs          # GET / — paginated post list
    ├── post.rs          # GET /post/:slug — single post
    ├── tag.rs           # GET /tag/:slug — posts by tag
    ├── search.rs        # GET /search?q= — search
    ├── react.rs         # POST /api/react — emoji reactions
    └── feed.rs          # GET /feed.xml — RSS feed
templates/               # Askama HTML templates
static/                  # CSS and HTMX
```

## Prerequisites

- [Rust](https://www.rustup.rs/) (1.85+)
- [Sanity account](https://www.sanity.io/) with a project
- [Redis](https://redis.io/) instance (local or Upstash/Redis Cloud)
- [Vercel CLI](https://vercel.com/docs/cli) (for deployment)
- [Docker](https://www.docker.com/) (for Vercel deployment)

## Local Development

1. **Clone and install dependencies:**
   ```bash
   git clone <your-repo-url>
   cd personal-blog
   ```

2. **Set up environment variables:**
   ```bash
   cp .env.example .env
   ```
   Fill in your Sanity project ID and Redis URL.

3. **Run the dev server:**
   ```bash
   cargo run
   ```
   Server starts at `http://localhost:3000`.

## Sanity Setup

1. **Initialize Sanity Studio** (separate directory):
   ```bash
   npx sanity@latest init
   ```

2. **Define content schemas** in `sanity.config.ts`:
   ```typescript
   // schemas/post.ts
   export default {
     name: 'post',
     type: 'document',
     fields: [
       { name: 'title', type: 'string' },
       { name: 'slug', type: 'slug', options: { source: 'title' } },
       { name: 'excerpt', type: 'text' },
       { name: 'body', type: 'array', of: [{ type: 'block' }, { type: 'image' }] },
       { name: 'tags', type: 'array', of: [{ type: 'reference', to: [{ type: 'tag' }] }] },
       { name: 'mainImage', type: 'image' },
       { name: 'publishedAt', type: 'datetime' },
     ],
   }
   ```

3. **Create a GROQ-powered query** in Sanity Studio's Vision tool to test.

4. **Deploy Studio:**
   ```bash
   npx sanity deploy
   ```

## Deployment (Vercel)

1. **Push to GitHub**

2. **Connect to Vercel:**
   ```bash
   vercel
   ```

3. **Set environment variables** in Vercel dashboard:
   - `SANITY_PROJECT_ID`
   - `SANITY_DATASET`
   - `SANITY_API_VERSION`
   - `REDIS_URL`

4. **Deploy:**
   ```bash
   vercel --prod
   ```

The `Dockerfile` builds a multi-stage image (~20MB final size). Vercel handles autoscaling and HTTPS automatically.

## Architecture

```
Browser → Vercel → Docker Container → Axum
                                         ├── Sanity API (content)
                                         └── Redis (cache)
```

- **Cache-through pattern:** Every request checks Redis first. On miss, fetches from Sanity, caches the result, then returns.
- **HTMX partials:** Search, pagination, and reactions return HTML fragments — HTMX swaps them into the DOM without a full page reload.
- **Portable Text:** Sanity's rich text is stored as structured JSON, rendered to HTML server-side by the `portable_text` module.

## License

MIT
