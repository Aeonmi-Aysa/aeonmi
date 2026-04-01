document.addEventListener('DOMContentLoaded', function() {
  const analyzeBtn = document.getElementById('analyze-btn');
  const summarizeBtn = document.getElementById('summarize-btn');
  const resultArea = document.getElementById('result-area');
  const resultText = document.getElementById('result-text');
  const backendStatus = document.getElementById('backend-status');

  // Check backend connection
  checkBackendStatus();

  analyzeBtn.addEventListener('click', function() {
    performAction('analyze');
  });

  summarizeBtn.addEventListener('click', function() {
    performAction('summarize');
  });

  function checkBackendStatus() {
    // Simulate checking localhost:5000
    fetch('http://localhost:5000/api/health')
      .then(response => {
        if (response.ok) {
          backendStatus.textContent = 'Connected';
          backendStatus.style.color = '#238636';
        } else {
          backendStatus.textContent = 'Error';
          backendStatus.style.color = '#DA3633';
        }
      })
      .catch(() => {
        backendStatus.textContent = 'Offline';
        backendStatus.style.color = '#8B949E';
      });
  }

  function performAction(action) {
    resultArea.classList.remove('hidden');
    resultText.textContent = 'Processing with Quantum AI...';

    // Get current tab
    chrome.tabs.query({active: true, currentWindow: true}, function(tabs) {
      const activeTab = tabs[0];
      
      // Simulate processing
      setTimeout(() => {
        if (action === 'analyze') {
          resultText.textContent = `Analyzed content of "${activeTab.title}". Quantum coherence detected in page structure. Optimization opportunities found: 3.`;
        } else {
          resultText.textContent = `Summary of "${activeTab.title}": This page contains information relevant to your current workflow. Key entities extracted and synced with QuantumForge database.`;
        }
      }, 1500);
    });
  }
});
