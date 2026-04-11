# How to Deploy Aeonmi Website to Netlify

## Current Status
✅ Website files updated and committed to git  
✅ All 4 pages complete (index, starter-kit, contact, blog)  
❌ NOT deployed to Netlify yet

---

## Option 1: Manual Deploy (5 minutes)

### Step 1: Create a zip of the website folder
```cmd
cd "C:/Users/wlwil/Desktop/Aeonmi Files/Aeonmi-aeonmi01/Aeonmi_Master"
tar -czf website_deploy.zip website
```

### Step 2: Upload to Netlify
1. Go to https://app.netlify.com
2. Log in to your account
3. Find your Aeonmi site (or click "Add new site" if first time)
4. Click "Deploys" tab
5. Drag `website_deploy.zip` into the upload area
6. Wait 30 seconds
7. ✅ Live at your-site.netlify.app

---

## Option 2: GitHub Auto-Deploy (Best for ongoing updates)

### Step 1: Push to GitHub
```cmd
cd "C:/Users/wlwil/Desktop/Aeonmi Files/Aeonmi-aeonmi01/Aeonmi_Master"
git push origin main
```

### Step 2: Link Netlify to GitHub
1. Go to https://app.netlify.com
2. Click "Add new site" → "Import an existing project"
3. Choose "GitHub"
4. Select your repository
5. Configure build settings:
   - **Base directory:** `website`
   - **Publish directory:** `website`
   - **Build command:** (leave empty)
6. Click "Deploy site"

### Step 3: Future updates
Every time you run:
```cmd
git add website/
git commit -m "Update website"
git push
```
→ Netlify automatically deploys in 30 seconds!

---

## Option 3: Use the Deploy Script

### Run this:
```cmd
cd "C:/Users/wlwil/Desktop/Aeonmi Files/Aeonmi-aeonmi01/Aeonmi_Master"
deploy_website.bat
```

This script:
1. Commits changes to git
2. Pushes to GitHub
3. Shows instructions for Netlify

---

## What Files Will Be Deployed

```
website/
├── index.html (Homepage - 15.6 KB)
├── starter-kit.html (Download page - 14.4 KB)
├── contact.html (Support - 5.4 KB)
├── download.html (Legacy - 8.4 KB)
├── examples.html (Legacy - 3.3 KB)
├── README.md (Docs - 2.3 KB)
└── blog/
    └── index.html (Blog homepage)
```

**Total size:** ~50 KB (ultra-fast loading)

---

## After Deployment

### Test these URLs:
- `https://your-site.netlify.app/` → Homepage
- `https://your-site.netlify.app/starter-kit.html` → Download page
- `https://your-site.netlify.app/contact.html` → Support
- `https://your-site.netlify.app/blog/` → Blog

### Verify:
- ✅ Starter Kit section shows on homepage
- ✅ Download button works on starter-kit.html
- ✅ All 4 quantum examples listed
- ✅ Pricing tiers visible
- ✅ Contact links work

---

## Custom Domain (Optional)

### If you own aeonmi.ai:
1. In Netlify: Site settings → Domain management
2. Add custom domain: `aeonmi.ai`
3. Add DNS records (Netlify provides them)
4. Wait 24-48 hours for propagation
5. ✅ Live at https://aeonmi.ai

---

## Troubleshooting

**Problem:** "Site not found"  
**Solution:** Check base directory is set to `website` in Netlify settings

**Problem:** "404 on /starter-kit.html"  
**Solution:** Make sure you deployed the `website/` folder, not the root

**Problem:** "Old content still showing"  
**Solution:** Clear browser cache or use incognito mode

---

## Quick Commands Reference

```cmd
# Manual deploy (create zip)
cd Aeonmi_Master
tar -czf website_deploy.zip website

# Auto-deploy (push to GitHub)
cd Aeonmi_Master
git add website/
git commit -m "Update website"
git push origin main

# Use deploy script
cd Aeonmi_Master
deploy_website.bat
```

---

## What Happens Next

Once deployed, users can:
1. Visit your website
2. Click "Download Starter Kit"
3. See the 4 quantum examples
4. View pricing tiers
5. Contact you for beta access

**You'll start getting beta requests immediately!**

---

## Recommendation

**Use Option 2 (GitHub Auto-Deploy)**

Why?
- One-time setup (10 minutes)
- Future updates are automatic
- No manual zip/upload needed
- Git history preserved
- Rollback capability

**Do it now:**
1. Run `git push origin main`
2. Link Netlify to GitHub
3. Done forever!

---

## Need Help?

Tell me:
- **"manual"** — I'll guide you through manual upload
- **"auto"** — I'll help set up GitHub auto-deploy
- **"stuck"** — Describe the error you're seeing