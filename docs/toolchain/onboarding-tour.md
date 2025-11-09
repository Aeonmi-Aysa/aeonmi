# Onboarding Tour System

Interactive guided tours and contextual help for the Aeonmi Shard Editor.

## Interactive Tour Framework

```javascript
// gui/static/js/onboarding-tour.js

class OnboardingTour {
    constructor(projectInfo) {
        this.projectInfo = projectInfo;
        this.currentStep = 0;
        this.steps = [];
        this.overlay = null;
        this.spotlight = null;
        this.tooltip = null;
        
        // Initialize tour steps based on project type
        this.initializeTour();
    }
    
    initializeTour() {
        const tourType = this.projectInfo.template || 'general';
        
        switch (tourType) {
            case 'basic-hello':
                this.steps = this.getHelloWorldTour();
                break;
            case 'grovers-search':
                this.steps = this.getGroversTour();
                break;
            case 'quantum-teleportation':
                this.steps = this.getTeleportationTour();
                break;
            default:
                this.steps = this.getGeneralTour();
        }
    }
    
    getGeneralTour() {
        return [
            {
                target: '.file-explorer',
                title: 'File Explorer',
                content: `
                    <p>Welcome to your quantum project! This is the file explorer where you can:</p>
                    <ul>
                        <li>Browse your project files</li>
                        <li>Create new quantum programs</li>
                        <li>Organize your code</li>
                    </ul>
                    <p>Your main quantum program is in <code>src/main.aeon</code>.</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.editor-container',
                title: 'Quantum Code Editor',
                content: `
                    <p>This is where you write your quantum programs! The editor provides:</p>
                    <ul>
                        <li><strong>Syntax highlighting</strong> for quantum operations</li>
                        <li><strong>Autocomplete</strong> for quantum functions</li>
                        <li><strong>Error detection</strong> as you type</li>
                        <li><strong>Quantum state visualization</strong> (coming up!)</li>
                    </ul>
                `,
                position: 'left',
                showNext: true
            },
            {
                target: '.toolbar',
                title: 'Toolbar',
                content: `
                    <p>Use these tools to work with your quantum code:</p>
                    <ul>
                        <li><span class="btn-icon">▶️</span> <strong>Run</strong> - Execute your quantum program</li>
                        <li><span class="btn-icon">🔧</span> <strong>Build</strong> - Compile to check for errors</li>
                        <li><span class="btn-icon">🧪</span> <strong>Test</strong> - Run quantum tests</li>
                        <li><span class="btn-icon">📊</span> <strong>Visualize</strong> - See quantum circuit diagrams</li>
                    </ul>
                `,
                position: 'bottom',
                showNext: true
            },
            {
                target: '.quantum-console',
                title: 'Quantum Console',
                content: `
                    <p>The console shows:</p>
                    <ul>
                        <li><strong>Program output</strong> - Results from your quantum computations</li>
                        <li><strong>Compilation messages</strong> - Helpful feedback from the compiler</li>
                        <li><strong>Learning insights</strong> - Educational explanations (in learning mode)</li>
                    </ul>
                    <p>Try running your first program to see it in action!</p>
                `,
                position: 'top',
                showNext: true,
                actionButton: {
                    text: 'Run Program',
                    action: () => this.runFirstProgram()
                }
            },
            {
                target: '.visualization-panel',
                title: 'Quantum Visualization',
                content: `
                    <p>Watch your quantum program come to life! This panel shows:</p>
                    <ul>
                        <li><strong>Quantum circuits</strong> - Visual representation of your quantum operations</li>
                        <li><strong>State evolution</strong> - How qubits change over time</li>
                        <li><strong>Measurement results</strong> - Probability distributions and outcomes</li>
                    </ul>
                `,
                position: 'left',
                showNext: true
            },
            {
                target: '.help-panel',
                title: 'Always Here to Help',
                content: `
                    <p>Need assistance? We've got you covered:</p>
                    <ul>
                        <li><span class="btn-icon">📚</span> <strong>Documentation</strong> - Complete language reference</li>
                        <li><span class="btn-icon">🎓</span> <strong>Tutorials</strong> - Step-by-step learning</li>
                        <li><span class="btn-icon">💡</span> <strong>Examples</strong> - Ready-to-run quantum programs</li>
                        <li><span class="btn-icon">💬</span> <strong>Community</strong> - Connect with other quantum programmers</li>
                    </ul>
                    <p>Click the help icon anytime you're stuck!</p>
                `,
                position: 'left',
                showNext: false,
                isLast: true
            }
        ];
    }
    
    getHelloWorldTour() {
        return [
            {
                target: '.file-explorer [data-file="src/main.aeon"]',
                title: 'Your First Quantum Program',
                content: `
                    <p>🎉 Welcome to quantum programming! Let's explore your first quantum program.</p>
                    <p>Click on <code>main.aeon</code> to open it in the editor.</p>
                `,
                position: 'right',
                showNext: true,
                prerequisite: () => this.waitForFileOpen('src/main.aeon')
            },
            {
                target: '.editor-container .code-line:nth-child(1)',
                title: 'Quantum Function Declaration',
                content: `
                    <p>This line declares a <strong>quantum function</strong>:</p>
                    <pre><code>quantum fn main() {</code></pre>
                    <p>The <code>quantum</code> keyword tells Aeonmi this function works with quantum data!</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.editor-container .code-line:has(.keyword:contains("qubit"))',
                title: 'Creating Your First Qubit',
                content: `
                    <p>Here we create a <strong>qubit</strong> - the fundamental unit of quantum information:</p>
                    <pre><code>let q = qubit(0);</code></pre>
                    <p>This creates a qubit in the |0⟩ state. Think of it as a quantum version of a bit that can be 0, 1, or both simultaneously!</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.editor-container .code-line:has(.function-call:contains("hadamard"))',
                title: 'Creating Quantum Superposition',
                content: `
                    <p>The Hadamard gate is magical:</p>
                    <pre><code>hadamard(q);</code></pre>
                    <p>It puts the qubit in <strong>superposition</strong> - simultaneously 0 AND 1!</p>
                    <p>This is what makes quantum computing special. 🌟</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.editor-container .code-line:has(.function-call:contains("measure"))',
                title: 'Quantum Measurement',
                content: `
                    <p>When we measure a quantum state:</p>
                    <pre><code>let result = measure(q);</code></pre>
                    <p>The superposition <strong>collapses</strong> randomly to either 0 or 1!</p>
                    <p>Each run gives a different result - true quantum randomness.</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.toolbar .run-button',
                title: 'Time to See Quantum Magic!',
                content: `
                    <p>Ready to run your first quantum program?</p>
                    <p>Click the <strong>Run</strong> button to execute your quantum code and see the random result!</p>
                    <p>Try running it multiple times - you'll get different outcomes each time. 🎲</p>
                `,
                position: 'bottom',
                showNext: true,
                actionButton: {
                    text: 'Run Quantum Program!',
                    action: () => this.runProgram()
                }
            },
            {
                target: '.quantum-console .output',
                title: 'Quantum Results',
                content: `
                    <p>🎊 Congratulations! You just ran your first quantum program!</p>
                    <p>The result you see is truly random - not pseudo-random like classical computers.</p>
                    <p><strong>Try this:</strong> Run the program several times and notice how the results change!</p>
                `,
                position: 'top',
                showNext: true,
                actionButton: {
                    text: 'Run Again',
                    action: () => this.runProgram()
                }
            },
            {
                target: '.visualization-panel',
                title: 'Visualizing Quantum States',
                content: `
                    <p>Look at the quantum circuit diagram! It shows:</p>
                    <ul>
                        <li><strong>H</strong> - The Hadamard gate creating superposition</li>
                        <li><strong>M</strong> - The measurement operation</li>
                        <li><strong>Lines</strong> - Quantum wires carrying information</li>
                    </ul>
                    <p>This visual helps you understand what your quantum program does!</p>
                `,
                position: 'left',
                showNext: true
            },
            {
                target: '.help-panel',
                title: 'Next Steps in Your Quantum Journey',
                content: `
                    <p>🚀 Amazing work! You've:</p>
                    <ul>
                        <li>✅ Created your first qubit</li>
                        <li>✅ Applied a quantum gate</li>
                        <li>✅ Observed quantum randomness</li>
                        <li>✅ Visualized quantum circuits</li>
                    </ul>
                    <p><strong>Ready for more?</strong> Try the <em>Quantum Entanglement</em> or <em>Grover's Search</em> tutorials!</p>
                `,
                position: 'left',
                showNext: false,
                isLast: true,
                actionButton: {
                    text: 'Browse Tutorials',
                    action: () => this.openTutorials()
                }
            }
        ];
    }
    
    getGroversTour() {
        return [
            {
                target: '.file-explorer',
                title: 'Grover\'s Quantum Search',
                content: `
                    <p>🔍 Welcome to Grover's Algorithm - the quantum search that's faster than any classical method!</p>
                    <p>You'll learn how to search unsorted databases with a <strong>quadratic speedup</strong>.</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.editor-container .function:has(.identifier:contains("grover"))',
                title: 'The Grover Algorithm Function',
                content: `
                    <p>This is the heart of Grover's algorithm:</p>
                    <pre><code>quantum fn grovers_search(database_size: usize, target: usize) -> usize</code></pre>
                    <p>It searches for a specific item in an unsorted database of any size!</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.editor-container .code-line:has(.function-call:contains("hadamard"))',
                title: 'Superposition Initialization',
                content: `
                    <p>Grover's starts by creating superposition over ALL possible states:</p>
                    <pre><code>hadamard(qubits);</code></pre>
                    <p>This means we're searching <strong>all possibilities simultaneously</strong>!</p>
                    <p>Classical computers must check items one by one. 🐌</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.editor-container .function:has(.identifier:contains("oracle"))',
                title: 'The Quantum Oracle',
                content: `
                    <p>The oracle is like a quantum detective 🕵️:</p>
                    <pre><code>quantum fn grover_oracle(qubits: &mut [Qubit], target: usize)</code></pre>
                    <p>It secretly "marks" the target item by flipping its amplitude.</p>
                    <p>This marking is invisible to us but crucial for the algorithm!</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.editor-container .function:has(.identifier:contains("diffusion"))',
                title: 'Amplitude Amplification',
                content: `
                    <p>The diffusion operator is the magic 🪄:</p>
                    <pre><code>quantum fn grover_diffusion(qubits: &mut [Qubit])</code></pre>
                    <p>It amplifies the probability of finding the marked item!</p>
                    <p>Each iteration makes the target more likely to be measured.</p>
                `,
                position: 'right',
                showNext: true
            },
            {
                target: '.toolbar .run-button',
                title: 'Test the Quantum Search',
                content: `
                    <p>Ready to see quantum speedup in action?</p>
                    <p>This will search a database of 16 items for a specific target.</p>
                    <p><strong>Classical:</strong> Up to 16 checks needed<br>
                       <strong>Quantum:</strong> Only ~4 iterations! ⚡</p>
                `,
                position: 'bottom',
                showNext: true,
                actionButton: {
                    text: 'Run Grover\'s Search!',
                    action: () => this.runProgram()
                }
            },
            {
                target: '.quantum-console .output',
                title: 'Quantum Search Results',
                content: `
                    <p>🎯 Amazing! Grover's algorithm found the target!</p>
                    <p>Notice how it succeeded with high probability in just a few iterations.</p>
                    <p>Try changing the target item and running again to see the quantum advantage!</p>
                `,
                position: 'top',
                showNext: true
            },
            {
                target: '.visualization-panel .circuit-diagram',
                title: 'Understanding the Quantum Circuit',
                content: `
                    <p>The circuit shows the complete Grover's algorithm:</p>
                    <ul>
                        <li><strong>Initialization:</strong> Hadamard gates create superposition</li>
                        <li><strong>Oracle:</strong> Marks the target (hidden operations)</li>
                        <li><strong>Diffusion:</strong> Amplifies target probability</li>
                        <li><strong>Repeat:</strong> Oracle + Diffusion iterations</li>
                        <li><strong>Measurement:</strong> High probability of measuring target</li>
                    </ul>
                `,
                position: 'left',
                showNext: true
            },
            {
                target: '.help-panel',
                title: 'Mastering Quantum Search',
                content: `
                    <p>🏆 Excellent! You understand Grover's algorithm!</p>
                    <p><strong>Key insights:</strong></p>
                    <ul>
                        <li>Quantum computers can search faster than classical ones</li>
                        <li>Superposition lets us check all items simultaneously</li>
                        <li>Amplitude amplification increases success probability</li>
                    </ul>
                    <p><strong>Next:</strong> Try quantum cryptography or Shor's algorithm!</p>
                `,
                position: 'left',
                showNext: false,
                isLast: true,
                actionButton: {
                    text: 'Explore More Algorithms',
                    action: () => this.openExamples()
                }
            }
        ];
    }
    
    start() {
        this.createOverlay();
        this.showStep(0);
    }
    
    createOverlay() {
        // Create dark overlay
        this.overlay = document.createElement('div');
        this.overlay.className = 'tour-overlay';
        this.overlay.innerHTML = `
            <div class="tour-controls">
                <button class="tour-skip" onclick="tour.skip()">Skip Tour</button>
                <div class="tour-progress">
                    <span class="tour-step-counter">1 of ${this.steps.length}</span>
                    <div class="tour-progress-bar">
                        <div class="tour-progress-fill" style="width: ${100/this.steps.length}%"></div>
                    </div>
                </div>
            </div>
        `;
        
        document.body.appendChild(this.overlay);
        
        // Create spotlight
        this.spotlight = document.createElement('div');
        this.spotlight.className = 'tour-spotlight';
        document.body.appendChild(this.spotlight);
        
        // Create tooltip
        this.tooltip = document.createElement('div');
        this.tooltip.className = 'tour-tooltip';
        document.body.appendChild(this.tooltip);
    }
    
    showStep(stepIndex) {
        if (stepIndex >= this.steps.length) {
            this.complete();
            return;
        }
        
        this.currentStep = stepIndex;
        const step = this.steps[stepIndex];
        
        // Wait for prerequisite if specified
        if (step.prerequisite) {
            step.prerequisite().then(() => {
                this.renderStep(step);
            });
        } else {
            this.renderStep(step);
        }
    }
    
    renderStep(step) {
        // Position spotlight on target element
        const target = document.querySelector(step.target);
        if (target) {
            this.positionSpotlight(target);
            this.positionTooltip(target, step);
        }
        
        // Update tooltip content
        this.tooltip.innerHTML = `
            <div class="tooltip-header">
                <h3>${step.title}</h3>
                <button class="tooltip-close" onclick="tour.skip()">×</button>
            </div>
            <div class="tooltip-content">
                ${step.content}
            </div>
            <div class="tooltip-footer">
                ${step.actionButton ? `
                    <button class="tour-action-btn" onclick="${step.actionButton.action.name}()">
                        ${step.actionButton.text}
                    </button>
                ` : ''}
                ${this.currentStep > 0 ? `
                    <button class="tour-prev-btn" onclick="tour.previousStep()">Previous</button>
                ` : ''}
                ${step.showNext ? `
                    <button class="tour-next-btn" onclick="tour.nextStep()">
                        ${step.isLast ? 'Finish' : 'Next'}
                    </button>
                ` : ''}
            </div>
        `;
        
        // Update progress
        this.updateProgress();
        
        // Add pulsing effect to target
        if (target) {
            target.classList.add('tour-highlight');
            setTimeout(() => {
                target.classList.remove('tour-highlight');
            }, 2000);
        }
    }
    
    positionSpotlight(target) {
        const rect = target.getBoundingClientRect();
        const padding = 10;
        
        this.spotlight.style.left = `${rect.left - padding}px`;
        this.spotlight.style.top = `${rect.top - padding}px`;
        this.spotlight.style.width = `${rect.width + padding * 2}px`;
        this.spotlight.style.height = `${rect.height + padding * 2}px`;
    }
    
    positionTooltip(target, step) {
        const rect = target.getBoundingClientRect();
        const tooltipRect = this.tooltip.getBoundingClientRect();
        
        let left, top;
        
        switch (step.position) {
            case 'right':
                left = rect.right + 20;
                top = rect.top + (rect.height - tooltipRect.height) / 2;
                break;
            case 'left':
                left = rect.left - tooltipRect.width - 20;
                top = rect.top + (rect.height - tooltipRect.height) / 2;
                break;
            case 'bottom':
                left = rect.left + (rect.width - tooltipRect.width) / 2;
                top = rect.bottom + 20;
                break;
            case 'top':
                left = rect.left + (rect.width - tooltipRect.width) / 2;
                top = rect.top - tooltipRect.height - 20;
                break;
            default:
                left = rect.right + 20;
                top = rect.top;
        }
        
        // Keep tooltip on screen
        left = Math.max(10, Math.min(left, window.innerWidth - tooltipRect.width - 10));
        top = Math.max(10, Math.min(top, window.innerHeight - tooltipRect.height - 10));
        
        this.tooltip.style.left = `${left}px`;
        this.tooltip.style.top = `${top}px`;
    }
    
    updateProgress() {
        const progress = ((this.currentStep + 1) / this.steps.length) * 100;
        const progressFill = document.querySelector('.tour-progress-fill');
        const stepCounter = document.querySelector('.tour-step-counter');
        
        if (progressFill) progressFill.style.width = `${progress}%`;
        if (stepCounter) stepCounter.textContent = `${this.currentStep + 1} of ${this.steps.length}`;
    }
    
    nextStep() {
        this.showStep(this.currentStep + 1);
    }
    
    previousStep() {
        this.showStep(this.currentStep - 1);
    }
    
    skip() {
        this.complete();
    }
    
    complete() {
        // Remove tour elements
        if (this.overlay) this.overlay.remove();
        if (this.spotlight) this.spotlight.remove();
        if (this.tooltip) this.tooltip.remove();
        
        // Show completion message
        this.showCompletionMessage();
        
        // Mark tour as completed for this project type
        this.markTourCompleted();
    }
    
    showCompletionMessage() {
        const completion = document.createElement('div');
        completion.className = 'tour-completion';
        completion.innerHTML = `
            <div class="completion-content">
                <div class="completion-icon">🎉</div>
                <h2>Tour Complete!</h2>
                <p>You're ready to explore quantum programming with Aeonmi.</p>
                <div class="completion-actions">
                    <button onclick="this.parentElement.parentElement.parentElement.remove()">
                        Continue Coding
                    </button>
                </div>
            </div>
        `;
        
        document.body.appendChild(completion);
        
        setTimeout(() => {
            completion.remove();
        }, 5000);
    }
    
    markTourCompleted() {
        const completedTours = JSON.parse(localStorage.getItem('aeonmi_completed_tours') || '[]');
        const tourId = `${this.projectInfo.template}_tour`;
        
        if (!completedTours.includes(tourId)) {
            completedTours.push(tourId);
            localStorage.setItem('aeonmi_completed_tours', JSON.stringify(completedTours));
        }
    }
    
    // Helper methods for tour actions
    async waitForFileOpen(filename) {
        return new Promise((resolve) => {
            const checkFileOpen = () => {
                const activeTab = document.querySelector('.editor-tab.active');
                if (activeTab && activeTab.textContent.includes(filename)) {
                    resolve();
                } else {
                    setTimeout(checkFileOpen, 100);
                }
            };
            checkFileOpen();
        });
    }
    
    runProgram() {
        // Trigger program execution
        const runButton = document.querySelector('.run-button');
        if (runButton) {
            runButton.click();
        }
    }
    
    runFirstProgram() {
        this.runProgram();
        // Auto-advance to next step after execution
        setTimeout(() => {
            this.nextStep();
        }, 2000);
    }
    
    openTutorials() {
        // Navigate to tutorials section
        window.showTutorials?.();
    }
    
    openExamples() {
        // Navigate to examples section
        window.showExamples?.();
    }
}

// Global tour instance
let tour = null;

// Auto-start tour for new projects
window.addEventListener('DOMContentLoaded', () => {
    const urlParams = new URLSearchParams(window.location.search);
    const newProject = urlParams.get('new_project');
    
    if (newProject) {
        // Wait for editor to load, then start tour
        setTimeout(() => {
            const projectInfo = JSON.parse(sessionStorage.getItem('new_project_info') || '{}');
            tour = new OnboardingTour(projectInfo);
            tour.start();
        }, 1000);
    }
});
```

This complete onboarding tour system provides:

1. **Context-Aware Tours** - Different tours for different project types
2. **Interactive Elements** - Clickable actions and guided interactions
3. **Visual Spotlights** - Highlights specific UI elements
4. **Progress Tracking** - Shows tour progress and allows navigation
5. **Educational Content** - Rich explanations with code examples
6. **Completion Tracking** - Remembers completed tours
7. **Flexible Positioning** - Adapts tooltip placement to screen space

The system integrates seamlessly with the project wizard and provides an engaging learning experience for new users.