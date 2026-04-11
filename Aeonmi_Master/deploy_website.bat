@echo off
echo ========================================
echo   DEPLOYING AEONMI WEBSITE TO NETLIFY
echo ========================================
echo.

cd /d "%~dp0"

echo Current directory: %CD%
echo.

echo Step 1: Check git status...
git status

echo.
echo Step 2: Stage website changes...
git add website/

echo.
echo Step 3: Commit changes...
git commit -m "Update website - %date% %time%"

echo.
echo Step 4: Push to GitHub...
git push origin main

echo.
echo ========================================
echo   GITHUB PUSH COMPLETE!
echo ========================================
echo.
echo NEXT STEPS:
echo.
echo Option A: Manual Deploy (if first time)
echo   1. Go to https://app.netlify.com
echo   2. Drag 'website' folder into Netlify
echo   3. Wait 30 seconds
echo   4. Your site is live!
echo.
echo Option B: Auto-Deploy (if already linked)
echo   - Netlify will auto-deploy in 30 seconds
echo   - Check: https://app.netlify.com
echo   - Look for new deploy in progress
echo.
echo Option C: Link GitHub (one-time setup)
echo   1. Go to https://app.netlify.com
echo   2. Add new site -^> Import from GitHub
echo   3. Select your repo
echo   4. Base directory: website
echo   5. Publish directory: website
echo   6. Deploy!
echo.
echo ========================================
echo.

pause