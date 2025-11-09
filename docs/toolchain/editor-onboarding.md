# Editor Onboarding Features

Welcome screens, project wizards, and guided learning paths for the Aeonmi Shard Editor.

## 🎉 Welcome Experience

### First-Time User Welcome Screen

When users first open the Shard Editor, they're greeted with an engaging welcome experience that helps them understand quantum programming and get started quickly.

```html
<!-- gui/static/welcome.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Welcome to Aeonmi - Quantum Programming Made Accessible</title>
    <link rel="stylesheet" href="css/welcome.css">
</head>
<body>
    <div class="welcome-container">
        <!-- Hero Section -->
        <section class="hero">
            <div class="quantum-animation">
                <div class="qubit-visualization"></div>
            </div>
            <h1>Welcome to <span class="brand">Aeonmi</span></h1>
            <p class="tagline">Your gateway to quantum programming</p>
            <p class="description">
                Build the future with quantum algorithms, secure cryptography, 
                and computational possibilities beyond classical computing.
            </p>
        </section>
        
        <!-- User Type Selection -->
        <section class="user-types">
            <h2>What brings you to quantum computing?</h2>
            <div class="user-cards">
                <div class="user-card" data-type="student">
                    <div class="card-icon">🎓</div>
                    <h3>Student</h3>
                    <p>Learning quantum computing concepts and theory</p>
                    <ul class="benefits">
                        <li>Interactive tutorials</li>
                        <li>Visual explanations</li>
                        <li>Step-by-step guidance</li>
                        <li>Educational examples</li>
                    </ul>
                    <button class="select-btn">Start Learning</button>
                </div>
                
                <div class="user-card" data-type="developer">
                    <div class="card-icon">👩‍💻</div>
                    <h3>Developer</h3>
                    <p>Building quantum applications and exploring algorithms</p>
                    <ul class="benefits">
                        <li>Ready-to-use templates</li>
                        <li>API documentation</li>
                        <li>Performance tools</li>
                        <li>Deployment guides</li>
                    </ul>
                    <button class="select-btn">Start Building</button>
                </div>
                
                <div class="user-card" data-type="researcher">
                    <div class="card-icon">🔬</div>
                    <h3>Researcher</h3>
                    <p>Implementing novel quantum algorithms and experiments</p>
                    <ul class="benefits">
                        <li>Advanced features</li>
                        <li>Benchmarking tools</li>
                        <li>Custom simulators</li>
                        <li>Publication support</li>
                    </ul>
                    <button class="select-btn">Start Research</button>
                </div>
                
                <div class="user-card" data-type="educator">
                    <div class="card-icon">🏫</div>
                    <h3>Educator</h3>
                    <p>Teaching quantum computing to students</p>
                    <ul class="benefits">
                        <li>Lesson plan templates</li>
                        <li>Interactive demos</li>
                        <li>Student progress tracking</li>
                        <li>Curriculum integration</li>
                    </ul>
                    <button class="select-btn">Start Teaching</button>
                </div>
            </div>
        </section>
        
        <!-- Quick Actions -->
        <section class="quick-actions">
            <h2>Or jump right in...</h2>
            <div class="action-buttons">
                <button class="action-btn primary" onclick="newProject()">
                    <span class="icon">📁</span>
                    Create New Project
                </button>
                <button class="action-btn secondary" onclick="openExample()">
                    <span class="icon">🎯</span>
                    Browse Examples
                </button>
                <button class="action-btn secondary" onclick="openTutorial()">
                    <span class="icon">📚</span>
                    Take Tutorial
                </button>
                <button class="action-btn secondary" onclick="openDocs()">
                    <span class="icon">📖</span>
                    Read Documentation
                </button>
            </div>
        </section>
        
        <!-- Recent Projects (shown if user has any) -->
        <section class="recent-projects" id="recentProjects" style="display: none;">
            <h2>Recent Projects</h2>
            <div class="project-list" id="projectList">
                <!-- Dynamically populated -->
            </div>
        </section>
    </div>
    
    <script src="js/welcome.js"></script>
</body>
</html>
```

### Interactive Project Wizard

```javascript
// gui/static/js/project-wizard.js

class ProjectWizard {
    constructor() {
        this.currentStep = 0;
        this.projectConfig = {
            name: '',
            type: '',
            template: '',
            features: [],
            userType: '',
        };
        this.steps = [
            'projectType',
            'projectDetails', 
            'features',
            'template',
            'confirmation'
        ];
    }
    
    async start(userType = null) {
        if (userType) {
            this.projectConfig.userType = userType;
        }
        
        this.showWizard();
        this.renderStep();
    }
    
    showWizard() {
        const wizardHTML = `
            <div class="wizard-overlay" id="projectWizard">
                <div class="wizard-container">
                    <div class="wizard-header">
                        <h2>New Quantum Project</h2>
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: 20%"></div>
                        </div>
                        <div class="step-indicator">
                            Step <span id="currentStep">1</span> of ${this.steps.length}
                        </div>
                    </div>
                    
                    <div class="wizard-content" id="wizardContent">
                        <!-- Dynamic content -->
                    </div>
                    
                    <div class="wizard-footer">
                        <button class="btn secondary" onclick="wizard.previousStep()" 
                                id="prevBtn" disabled>Previous</button>
                        <button class="btn primary" onclick="wizard.nextStep()" 
                                id="nextBtn">Next</button>
                        <button class="btn success" onclick="wizard.createProject()" 
                                id="createBtn" style="display: none;">Create Project</button>
                    </div>
                </div>
            </div>
        `;
        
        document.body.insertAdjacentHTML('beforeend', wizardHTML);
    }
    
    renderStep() {
        const stepName = this.steps[this.currentStep];
        const content = document.getElementById('wizardContent');
        
        switch (stepName) {
            case 'projectType':
                content.innerHTML = this.renderProjectTypeStep();
                break;
            case 'projectDetails':
                content.innerHTML = this.renderProjectDetailsStep();
                break;
            case 'features':
                content.innerHTML = this.renderFeaturesStep();
                break;
            case 'template':
                content.innerHTML = this.renderTemplateStep();
                break;
            case 'confirmation':
                content.innerHTML = this.renderConfirmationStep();
                break;
        }
        
        this.updateProgress();
        this.updateButtons();
    }
    
    renderProjectTypeStep() {
        const types = this.getProjectTypes();
        
        return `
            <div class="step-content">
                <h3>What type of quantum project are you creating?</h3>
                <div class="project-types">
                    ${types.map(type => `
                        <div class="project-type-card ${this.projectConfig.type === type.id ? 'selected' : ''}" 
                             onclick="wizard.selectProjectType('${type.id}')">
                            <div class="type-icon">${type.icon}</div>
                            <h4>${type.name}</h4>
                            <p>${type.description}</p>
                            <div class="type-features">
                                ${type.features.map(f => `<span class="feature-tag">${f}</span>`).join('')}
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    }
    
    renderProjectDetailsStep() {
        return `
            <div class="step-content">
                <h3>Project Details</h3>
                <div class="form-group">
                    <label for="projectName">Project Name</label>
                    <input type="text" id="projectName" 
                           value="${this.projectConfig.name}"
                           placeholder="my-quantum-project"
                           onchange="wizard.updateProjectName(this.value)">
                    <small>Use lowercase letters, numbers, and hyphens</small>
                </div>
                
                <div class="form-group">
                    <label for="projectDescription">Description (optional)</label>
                    <textarea id="projectDescription" 
                              placeholder="A brief description of your quantum project..."
                              onchange="wizard.updateProjectDescription(this.value)"></textarea>
                </div>
                
                <div class="form-group">
                    <label>Programming Experience</label>
                    <div class="radio-group">
                        <label class="radio-label">
                            <input type="radio" name="experience" value="beginner" 
                                   onchange="wizard.updateExperience(this.value)">
                            Beginner - New to programming
                        </label>
                        <label class="radio-label">
                            <input type="radio" name="experience" value="intermediate" 
                                   onchange="wizard.updateExperience(this.value)">
                            Intermediate - Some programming experience
                        </label>
                        <label class="radio-label">
                            <input type="radio" name="experience" value="advanced" 
                                   onchange="wizard.updateExperience(this.value)">
                            Advanced - Experienced programmer
                        </label>
                    </div>
                </div>
            </div>
        `;
    }
    
    renderFeaturesStep() {
        const features = this.getAvailableFeatures();
        
        return `
            <div class="step-content">
                <h3>Additional Features</h3>
                <p>Select features to include in your project:</p>
                
                <div class="features-grid">
                    ${features.map(feature => `
                        <label class="feature-checkbox">
                            <input type="checkbox" value="${feature.id}"
                                   ${this.projectConfig.features.includes(feature.id) ? 'checked' : ''}
                                   onchange="wizard.toggleFeature('${feature.id}')">
                            <div class="feature-card">
                                <div class="feature-icon">${feature.icon}</div>
                                <h4>${feature.name}</h4>
                                <p>${feature.description}</p>
                            </div>
                        </label>
                    `).join('')}
                </div>
            </div>
        `;
    }
    
    renderTemplateStep() {
        const templates = this.getTemplatesForType(this.projectConfig.type);
        
        return `
            <div class="step-content">
                <h3>Choose a Template</h3>
                <div class="templates-grid">
                    ${templates.map(template => `
                        <div class="template-card ${this.projectConfig.template === template.id ? 'selected' : ''}"
                             onclick="wizard.selectTemplate('${template.id}')">
                            <div class="template-preview">
                                <img src="${template.preview}" alt="${template.name} preview">
                            </div>
                            <h4>${template.name}</h4>
                            <p>${template.description}</p>
                            <div class="template-info">
                                <span class="difficulty ${template.difficulty}">${template.difficulty}</span>
                                <span class="estimated-time">${template.estimatedTime}</span>
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    }
    
    renderConfirmationStep() {
        const selectedType = this.getProjectTypes().find(t => t.id === this.projectConfig.type);
        const selectedTemplate = this.getTemplatesForType(this.projectConfig.type)
                                     .find(t => t.id === this.projectConfig.template);
        
        return `
            <div class="step-content">
                <h3>Ready to Create Your Project!</h3>
                
                <div class="project-summary">
                    <div class="summary-section">
                        <h4>Project Information</h4>
                        <table class="summary-table">
                            <tr><td>Name:</td><td><strong>${this.projectConfig.name}</strong></td></tr>
                            <tr><td>Type:</td><td>${selectedType?.name}</td></tr>
                            <tr><td>Template:</td><td>${selectedTemplate?.name}</td></tr>
                        </table>
                    </div>
                    
                    ${this.projectConfig.features.length > 0 ? `
                        <div class="summary-section">
                            <h4>Features</h4>
                            <div class="selected-features">
                                ${this.projectConfig.features.map(f => {
                                    const feature = this.getAvailableFeatures().find(feat => feat.id === f);
                                    return `<span class="feature-tag">${feature?.name}</span>`;
                                }).join('')}
                            </div>
                        </div>
                    ` : ''}
                    
                    <div class="summary-section">
                        <h4>What happens next?</h4>
                        <ol class="next-steps">
                            <li>Create project structure with selected template</li>
                            <li>Install required dependencies</li>
                            <li>Open project in the editor</li>
                            <li>Show getting started guide</li>
                        </ol>
                    </div>
                </div>
                
                <div class="confirmation-message">
                    <div class="message-icon">✨</div>
                    <p>Your quantum programming journey is about to begin!</p>
                </div>
            </div>
        `;
    }
    
    getProjectTypes() {
        const baseTypes = [
            {
                id: 'hello-world',
                name: 'Hello Quantum World',
                icon: '👋',
                description: 'Your first quantum program with basic qubit operations',
                features: ['Tutorial', 'Examples', 'Documentation']
            },
            {
                id: 'algorithm',
                name: 'Quantum Algorithm',
                icon: '🧮',
                description: 'Implement famous quantum algorithms like Grover\'s search',
                features: ['Algorithm Templates', 'Benchmarking', 'Visualization']
            },
            {
                id: 'cryptography',
                name: 'Quantum Cryptography',
                icon: '🔐',
                description: 'Secure communication with quantum key distribution',
                features: ['BB84 Protocol', 'Security Analysis', 'Network Simulation']
            },
            {
                id: 'simulation',
                name: 'Quantum Simulation',
                icon: '⚛️',
                description: 'Simulate physical quantum systems and chemistry',
                features: ['VQE', 'Hamiltonian Simulation', 'Molecular Modeling']
            },
            {
                id: 'machine-learning',
                name: 'Quantum ML',
                icon: '🤖',
                description: 'Quantum machine learning and optimization',
                features: ['QML Algorithms', 'Data Encoding', 'Hybrid Training']
            },
            {
                id: 'game',
                name: 'Quantum Game',
                icon: '🎮',
                description: 'Interactive games using quantum mechanics',
                features: ['Game Logic', 'UI Components', 'Quantum Mechanics']
            }
        ];
        
        // Filter based on user type
        if (this.projectConfig.userType === 'student') {
            return baseTypes.filter(t => ['hello-world', 'algorithm', 'game'].includes(t.id));
        } else if (this.projectConfig.userType === 'developer') {
            return baseTypes.filter(t => !['hello-world'].includes(t.id));
        }
        
        return baseTypes;
    }
    
    getAvailableFeatures() {
        return [
            {
                id: 'documentation',
                name: 'Documentation',
                icon: '📚',
                description: 'Generate API docs and user guides'
            },
            {
                id: 'testing',
                name: 'Testing Framework',
                icon: '🧪',
                description: 'Unit tests and quantum property testing'
            },
            {
                id: 'benchmarking',
                name: 'Benchmarking',
                icon: '⏱️',
                description: 'Performance measurement and comparison'
            },
            {
                id: 'visualization',
                name: 'Visualization',
                icon: '📊',
                description: 'Quantum state and circuit visualization'
            },
            {
                id: 'gui',
                name: 'User Interface',
                icon: '🖥️',
                description: 'Web-based or desktop user interface'
            },
            {
                id: 'deployment',
                name: 'Deployment',
                icon: '🚀',
                description: 'Docker containers and cloud deployment'
            }
        ];
    }
    
    getTemplatesForType(type) {
        const templates = {
            'hello-world': [
                {
                    id: 'basic-hello',
                    name: 'Basic Hello Quantum',
                    description: 'Simple qubit creation and measurement',
                    difficulty: 'beginner',
                    estimatedTime: '10 minutes',
                    preview: '/static/images/templates/hello-basic.png'
                },
                {
                    id: 'interactive-hello',
                    name: 'Interactive Tutorial',
                    description: 'Step-by-step guided introduction',
                    difficulty: 'beginner',
                    estimatedTime: '30 minutes',
                    preview: '/static/images/templates/hello-interactive.png'
                }
            ],
            'algorithm': [
                {
                    id: 'grovers-search',
                    name: 'Grover\'s Search',
                    description: 'Quantum database search algorithm',
                    difficulty: 'intermediate',
                    estimatedTime: '1 hour',
                    preview: '/static/images/templates/grovers.png'
                },
                {
                    id: 'quantum-teleportation',
                    name: 'Quantum Teleportation',
                    description: 'Transfer quantum states using entanglement',
                    difficulty: 'intermediate',
                    estimatedTime: '45 minutes',
                    preview: '/static/images/templates/teleportation.png'
                },
                {
                    id: 'shors-algorithm',
                    name: 'Shor\'s Algorithm',
                    description: 'Quantum integer factorization',
                    difficulty: 'advanced',
                    estimatedTime: '2 hours',
                    preview: '/static/images/templates/shors.png'
                }
            ],
            // ... more templates for other types
        };
        
        return templates[type] || [];
    }
    
    // Event handlers
    selectProjectType(typeId) {
        this.projectConfig.type = typeId;
        
        // Update UI
        document.querySelectorAll('.project-type-card').forEach(card => {
            card.classList.remove('selected');
        });
        document.querySelector(`[onclick="wizard.selectProjectType('${typeId}')"]`)
                ?.classList.add('selected');
                
        this.validateCurrentStep();
    }
    
    updateProjectName(name) {
        this.projectConfig.name = name.toLowerCase()
                                      .replace(/[^a-z0-9-]/g, '-')
                                      .replace(/-+/g, '-')
                                      .replace(/^-|-$/g, '');
        
        // Update the input field with cleaned name
        const input = document.getElementById('projectName');
        if (input && input.value !== this.projectConfig.name) {
            input.value = this.projectConfig.name;
        }
        
        this.validateCurrentStep();
    }
    
    toggleFeature(featureId) {
        const index = this.projectConfig.features.indexOf(featureId);
        if (index > -1) {
            this.projectConfig.features.splice(index, 1);
        } else {
            this.projectConfig.features.push(featureId);
        }
    }
    
    selectTemplate(templateId) {
        this.projectConfig.template = templateId;
        
        // Update UI
        document.querySelectorAll('.template-card').forEach(card => {
            card.classList.remove('selected');
        });
        document.querySelector(`[onclick="wizard.selectTemplate('${templateId}')"]`)
                ?.classList.add('selected');
                
        this.validateCurrentStep();
    }
    
    validateCurrentStep() {
        const stepName = this.steps[this.currentStep];
        let isValid = false;
        
        switch (stepName) {
            case 'projectType':
                isValid = !!this.projectConfig.type;
                break;
            case 'projectDetails':
                isValid = !!this.projectConfig.name && this.projectConfig.name.length > 0;
                break;
            case 'features':
                isValid = true; // Features are optional
                break;
            case 'template':
                isValid = !!this.projectConfig.template;
                break;
            case 'confirmation':
                isValid = true;
                break;
        }
        
        this.updateButtons();
        return isValid;
    }
    
    updateProgress() {
        const progress = ((this.currentStep + 1) / this.steps.length) * 100;
        const progressBar = document.querySelector('.progress-fill');
        const stepIndicator = document.getElementById('currentStep');
        
        if (progressBar) progressBar.style.width = `${progress}%`;
        if (stepIndicator) stepIndicator.textContent = this.currentStep + 1;
    }
    
    updateButtons() {
        const prevBtn = document.getElementById('prevBtn');
        const nextBtn = document.getElementById('nextBtn');
        const createBtn = document.getElementById('createBtn');
        const isLastStep = this.currentStep === this.steps.length - 1;
        const isValid = this.validateCurrentStep();
        
        if (prevBtn) prevBtn.disabled = this.currentStep === 0;
        if (nextBtn) {
            nextBtn.style.display = isLastStep ? 'none' : 'inline-block';
            nextBtn.disabled = !isValid;
        }
        if (createBtn) {
            createBtn.style.display = isLastStep ? 'inline-block' : 'none';
            createBtn.disabled = !isValid;
        }
    }
    
    nextStep() {
        if (this.currentStep < this.steps.length - 1 && this.validateCurrentStep()) {
            this.currentStep++;
            this.renderStep();
        }
    }
    
    previousStep() {
        if (this.currentStep > 0) {
            this.currentStep--;
            this.renderStep();
        }
    }
    
    async createProject() {
        try {
            // Show loading state
            this.showLoading();
            
            // Create the project
            const response = await fetch('/api/projects/create', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(this.projectConfig)
            });
            
            if (!response.ok) {
                throw new Error('Failed to create project');
            }
            
            const result = await response.json();
            
            // Close wizard
            this.closeWizard();
            
            // Open the new project
            await this.openProject(result.projectPath);
            
            // Show success message and onboarding
            this.showSuccessMessage(result);
            
        } catch (error) {
            console.error('Error creating project:', error);
            this.showError('Failed to create project. Please try again.');
        }
    }
    
    showLoading() {
        const content = document.getElementById('wizardContent');
        content.innerHTML = `
            <div class="loading-state">
                <div class="spinner"></div>
                <h3>Creating your quantum project...</h3>
                <p>Setting up project structure, installing dependencies, and preparing examples.</p>
            </div>
        `;
        
        // Disable all buttons
        document.querySelectorAll('.wizard-footer button').forEach(btn => {
            btn.disabled = true;
        });
    }
    
    closeWizard() {
        const wizard = document.getElementById('projectWizard');
        if (wizard) {
            wizard.remove();
        }
    }
    
    async openProject(projectPath) {
        // Load project files and switch to editor view
        await fetch('/api/projects/open', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ path: projectPath })
        });
        
        // Refresh the file explorer
        if (window.fileExplorer) {
            window.fileExplorer.refresh();
        }
        
        // Switch to editor tab
        window.hideWelcome?.();
    }
    
    showSuccessMessage(result) {
        // Show a congratulations message with next steps
        const toast = document.createElement('div');
        toast.className = 'success-toast';
        toast.innerHTML = `
            <div class="toast-content">
                <div class="toast-icon">🎉</div>
                <div class="toast-message">
                    <strong>Project created successfully!</strong>
                    <p>Your quantum project "${result.projectName}" is ready to go.</p>
                </div>
            </div>
        `;
        
        document.body.appendChild(toast);
        
        // Auto-remove after 5 seconds
        setTimeout(() => {
            toast.remove();
        }, 5000);
        
        // Start onboarding tour for the new project
        setTimeout(() => {
            this.startOnboardingTour(result);
        }, 1000);
    }
    
    startOnboardingTour(projectInfo) {
        // Initialize the onboarding tour based on project type
        if (window.OnboardingTour) {
            new OnboardingTour(projectInfo).start();
        }
    }
}

// Initialize wizard
const wizard = new ProjectWizard();
```

This comprehensive onboarding system provides:

1. **Welcome Screen** - Engaging first-time user experience with user type selection
2. **Project Wizard** - Step-by-step project creation with templates and features
3. **User Type Customization** - Different experiences for students, developers, researchers, and educators
4. **Interactive Templates** - Pre-built project templates for common quantum applications
5. **Feature Selection** - Optional components users can add to their projects
6. **Progress Tracking** - Visual progress indicators and validation
7. **Success Flow** - Confirmation and smooth transition to the editor

The system integrates seamlessly with the Shard Editor and provides a smooth onboarding experience for users of all backgrounds.