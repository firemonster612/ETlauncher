/** Onboarding wizard steps */
export type OnboardingStep =
	| 'welcome'
	| 'login'
	| 'theme'
	| 'font'
	| 'curseforge'
	| 'features'
	| 'complete';

const STEPS: OnboardingStep[] = [
	'welcome',
	'login',
	'theme',
	'font',
	'curseforge',
	'features',
	'complete',
];

/** Create the onboarding store */
function createOnboardingStore() {
	// State
	let isOpen = $state(false);
	let currentStep = $state<OnboardingStep>('welcome');
	let loginCompleted = $state(false);

	return {
		// Getters
		get isOpen() {
			return isOpen;
		},
		get currentStep() {
			return currentStep;
		},
		get currentStepIndex() {
			return STEPS.indexOf(currentStep);
		},
		get totalSteps() {
			return STEPS.length;
		},
		get loginCompleted() {
			return loginCompleted;
		},
		get isFirstStep() {
			return currentStep === STEPS[0];
		},
		get isLastStep() {
			return currentStep === STEPS[STEPS.length - 1];
		},

		/** Start the onboarding wizard */
		start() {
			isOpen = true;
			currentStep = 'welcome';
			loginCompleted = false;
		},

		/** Go to the next step */
		next() {
			const currentIndex = STEPS.indexOf(currentStep);
			if (currentIndex < STEPS.length - 1) {
				currentStep = STEPS[currentIndex + 1];
			}
		},

		/** Go to the previous step */
		back() {
			const currentIndex = STEPS.indexOf(currentStep);
			if (currentIndex > 0) {
				currentStep = STEPS[currentIndex - 1];
			}
		},

		/** Skip to the completion step */
		skip() {
			currentStep = 'complete';
		},

		/** Mark login as completed */
		setLoginCompleted(completed: boolean) {
			loginCompleted = completed;
		},

		/** Complete the onboarding and close the wizard */
		complete() {
			isOpen = false;
			currentStep = 'welcome';
			loginCompleted = false;
		},

		/** Close the wizard without completing (e.g., user dismissed) */
		close() {
			isOpen = false;
		},
	};
}

/** Global onboarding store instance */
export const onboardingStore = createOnboardingStore();
