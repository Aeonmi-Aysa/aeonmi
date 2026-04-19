// Background service worker for QuantumForge Extension

chrome.runtime.onInstalled.addListener(() => {
  console.log('QuantumForge Assistant installed.');
});

// Listen for messages from popup or content scripts
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.type === 'ANALYZE_CONTENT') {
    // Perform analysis logic here
    console.log('Analyzing content:', request.payload);
    sendResponse({status: 'success', data: 'Analysis complete'});
  }
  return true;
});
