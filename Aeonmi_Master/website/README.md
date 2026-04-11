# Aeonmi Website

**Live site:** https://aeonmi.ai (when deployed)

## Structure

```
website/
├── index.html          — Homepage
├── starter-kit.html    — Starter kit download page
├── contact.html        — Contact information
├── blog/
│   └── index.html      — Blog posts
└── README.md           — This file
```

## Pages

### Homepage (`index.html`)
- Hero section with CTA buttons
- Feature grid (6 key features)
- Code example (Bell state)
- Download section
- Roadmap
- Latest blog post preview

### Starter Kit (`starter-kit.html`)
- What's included (4 examples)
- Quick start guide
- Example code
- Pricing tiers (Free/Pro/Enterprise)
- FAQ
- Download instructions

### Contact (`contact.html`)
- Discord link
- Email addresses (support, academic, enterprise, beta)
- Response time expectations

### Blog (`blog/index.html`)
- Latest posts
- Starter kit announcement
- Introduction post

## Deployment

### Option 1: GitHub Pages (Free)
```bash
# Create gh-pages branch
git checkout -b gh-pages
git add website/*
git commit -m "Deploy website"
git push origin gh-pages

# Enable GitHub Pages in repo settings
# Settings → Pages → Source: gh-pages branch
```

### Option 2: Netlify (Free)
1. Sign up at netlify.com
2. Connect GitHub repo
3. Build settings:
   - Build command: (none)
   - Publish directory: `website/`
4. Deploy

### Option 3: Vercel (Free)
1. Sign up at vercel.com
2. Import GitHub repo
3. Root directory: `website/`
4. Deploy

### Option 4: Custom Domain
1. Buy domain (aeonmi.ai) from Namecheap/GoDaddy
2. Point DNS to hosting provider
3. Add SSL certificate (free with Let's Encrypt)

## Local Testing

```bash
# Python simple server
cd website
python -m http.server 8000

# Open http://localhost:8000
```

## Updates

To update the website:
1. Edit HTML files in `website/` folder
2. Test locally
3. Commit and push to GitHub
4. Deployment happens automatically (if using CI/CD)

## Analytics (Optional)

Add to `<head>` of each page:

```html
<!-- Google Analytics -->
<script async src="https://www.googletagmanager.com/gtag/js?id=G-XXXXXXXXXX"></script>
<script>
  window.dataLayer = window.dataLayer || [