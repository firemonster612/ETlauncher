/** Tutorial step definition */
export interface TutorialStep {
	id: string;
	targetSelector?: string; // Optional - some steps don't have a target (like login)
	title: string;
	description: string;
	position: 'top' | 'bottom' | 'left' | 'right' | 'center';
	navigateTo?: string; // Optional route to navigate to before showing this step
	type?: 'spotlight' | 'login'; // Step type - default is spotlight
	onEnter?: () => void | Promise<void>; // Optional hook to prep UI for this step
}

/** Tutorial covers both modpacks and custom instances in a single flow */

/** Complete tutorial steps - covers both modpacks and custom instances */
const TUTORIAL_STEPS: TutorialStep[] = [
	// Step 1: Login
	{
		id: 'login',
		title: 'Sign In',
		description: 'Sign in with your Microsoft account to play Minecraft',
		position: 'center',
		type: 'login',
	},
	// Step 2-4: Modpacks flow
	{
		id: 'sidebar-modpacks',
		targetSelector: "[data-tutorial='sidebar-modpacks']",
		title: 'Browse Modpacks',
		description:
			'The easiest way to start! Browse thousands of modpacks from Modrinth and CurseForge',
		position: 'right',
		navigateTo: '/instances',
	},
	{
		id: 'modpack-search',
		targetSelector: "[data-tutorial='modpack-search']",
		title: 'Search & Filter',
		description: 'Search for modpacks by name, or filter by Minecraft version and mod loader',
		position: 'bottom',
		navigateTo: '/modpacks',
	},
	{
		id: 'modpack-card',
		targetSelector: "[data-tutorial='modpack-card']",
		title: 'Install a Modpack',
		description:
			'Click any modpack to see details, then click Install to create a new instance with all mods included',
		position: 'bottom',
	},
	// Step 5-7: Custom instance flow
	{
		id: 'sidebar-instances',
		targetSelector: "[data-tutorial='sidebar-instances']",
		title: 'Manage Instances',
		description:
			"Now let's see how to create a custom instance. This is where all your Minecraft installations live",
		position: 'right',
	},
	{
		id: 'new-instance-btn',
		targetSelector: "[data-tutorial='new-instance-btn']",
		title: 'Create Custom Instance',
		description: 'Click New to create a vanilla or modded instance from scratch',
		position: 'bottom',
		navigateTo: '/instances',
	},
	{
		id: 'instance-info',
		targetSelector: "[data-tutorial='instance-version-loader']",
		title: 'Pick Version & Loader',
		description:
			'Choose your Minecraft version and loader (Fabric, Forge, etc.) before creating the instance',
		position: 'bottom',
		onEnter: () => {
			const newButton = document.querySelector(
				"[data-tutorial='new-instance-btn']"
			) as HTMLButtonElement | null;
			newButton?.click();
		},
	},
	{
		id: 'instance-settings-btn',
		targetSelector: "[data-tutorial='instance-settings-btn']",
		title: 'Open Instance Settings',
		description: 'Open settings for an existing instance to manage updates and loaders',
		position: 'bottom',
		navigateTo: '/instances',
		onEnter: async () => {
			const createCancel = document.querySelector(
				"[data-tutorial='create-cancel']"
			) as HTMLButtonElement | null;
			createCancel?.click();
			const settingsButton = (await ensureTutorialInstance()) as HTMLButtonElement | null;
			settingsButton?.click();
		},
	},
	{
		id: 'instance-update',
		targetSelector: "[data-tutorial='instance-update-button']",
		title: 'Update Version or Loader',
		description:
			'Use Check for Updates to bump Minecraft versions or reinstall loaders for this instance',
		position: 'top',
		navigateTo: '/instances',
		onEnter: async () => {
			const createCancel = document.querySelector(
				"[data-tutorial='create-cancel']"
			) as HTMLButtonElement | null;
			createCancel?.click();
			const settingsButton = (await ensureTutorialInstance()) as HTMLButtonElement | null;
			settingsButton?.click();

			const updateButton = (await waitForSelector(
				"[data-tutorial='instance-update-button']",
				20,
				150
			)) as HTMLElement | null;
			const sheetContent = document.querySelector(
				"[data-slot='sheet-content']"
			) as HTMLElement | null;
			if (updateButton && sheetContent) {
				const buttonTop =
					updateButton.getBoundingClientRect().top -
					sheetContent.getBoundingClientRect().top +
					sheetContent.scrollTop;
				const targetTop = Math.max(0, buttonTop - sheetContent.clientHeight / 2);
				sheetContent.scrollTo({ top: targetTop, behavior: 'auto' });
				updateButton.focus({ preventScroll: true });
			}
		},
	},
	{
		id: 'content-browser',
		targetSelector: "[data-tutorial='content-browser-btn']",
		title: 'Open Content Browser',
		description: 'Add mods, shaders, or resource packs to any instance with one click',
		position: 'bottom',
		navigateTo: '/instances',
		onEnter: async () => {
			const settingsClose = document.querySelector(
				"[data-slot='sheet-close']"
			) as HTMLButtonElement | null;
			settingsClose?.click();
			const contentButton = (await ensureTutorialInstance()) as HTMLButtonElement | null;
			contentButton?.click();
		},
	},
	{
		id: 'content-browser-types',
		targetSelector: "[data-tutorial='content-browser-types']",
		title: 'Content Types',
		description:
			'Choose from mods, shaders, or resource packs - search online and install directly to your instance',
		position: 'bottom',
		onEnter: () => {
			const contentButton = document.querySelector(
				"[data-tutorial='content-browser-btn']"
			) as HTMLButtonElement | null;
			contentButton?.click();
		},
	},
	// Step 10: Completion
	{
		id: 'ready',
		targetSelector: "[data-tutorial='sidebar-instances']",
		title: "You're All Set!",
		description:
			'Your instances appear here. Click the play button to launch, update versions, or customize with content!',
		position: 'right',
	},
];

// Navigation function - will be set by the layout component
let navigateFunction: ((path: string) => void) | null = null;

// Navigation timing
const NAVIGATION_START_DELAY_MS = 300;
const NAVIGATION_RETRY_MS = 200;
const NAVIGATION_MAX_ATTEMPTS = 30;

// Utility: wait for selector with retries
const waitForSelector = async (
	selector: string,
	attempts = 30,
	delayMs = 200
): Promise<HTMLElement | null> => {
	for (let i = 0; i < attempts; i++) {
		const el = document.querySelector(selector) as HTMLElement | null;
		if (el) return el;
		await new Promise((resolve) => setTimeout(resolve, delayMs));
	}
	return null;
};

// Utility: ensure we have at least one instance for later steps
const ensureTutorialInstance = async () => {
	const existing = document.querySelector(
		"[data-tutorial='instance-settings-btn']"
	) as HTMLElement | null;
	if (existing) return existing;

	const newButton = await waitForSelector("[data-tutorial='new-instance-btn']");
	newButton?.dispatchEvent(new Event('click', { bubbles: true }));

	const nameInput = (await waitForSelector('input#name')) as HTMLInputElement | null;
	if (nameInput) {
		nameInput.value = 'Tutorial Instance';
		nameInput.dispatchEvent(new Event('input', { bubbles: true }));
	}

	const createButton = (await waitForSelector(
		"[data-tutorial='instance-create']"
	)) as HTMLButtonElement | null;
	createButton?.click();

	return waitForSelector("[data-tutorial='instance-settings-btn']", 40, 250);
};

/** Create the tutorial store */
function createTutorialStore() {
	// State
	let isActive = $state(false);
	let showWelcome = $state(false);
	let currentStepIndex = $state(0);
	let steps = $state<TutorialStep[]>([]);
	let targetRect = $state<DOMRect | null>(null);

	// Get current step
	const getCurrentStep = () => {
		if (currentStepIndex >= 0 && currentStepIndex < steps.length) {
			return steps[currentStepIndex];
		}
		return null;
	};

	// Update target element position
	const updateTargetPosition = () => {
		const step = getCurrentStep();
		if (!step || !step.targetSelector) {
			targetRect = null;
			return;
		}

		const element = document.querySelector(step.targetSelector);
		if (element) {
			targetRect = element.getBoundingClientRect();
		} else {
			// Element not found - might need to navigate or wait
			targetRect = null;
		}
	};

	// Navigate and then update position after navigation completes
	const runStepPreparation = async (step: TutorialStep | null, attempt = 0) => {
		if (step?.onEnter) {
			try {
				await step.onEnter();
			} catch (error) {
				console.error('Tutorial step preparation failed', error);
			}
		}
		requestAnimationFrame(() => {
			updateTargetPosition();

			if (step?.onEnter && step.targetSelector) {
				const targetExists = document.querySelector(step.targetSelector);
				if (!targetExists && attempt < 3) {
					setTimeout(() => runStepPreparation(step, attempt + 1), NAVIGATION_RETRY_MS);
				}
			}
		});
	};

	const navigateAndUpdate = (path: string, step: TutorialStep | null) => {
		if (!navigateFunction) return;

		navigateFunction(path);

		let attempts = 0;
		let intervalId: number | null = null;
		let prepared = false;

		const tryUpdate = async () => {
			attempts++;

			if (!prepared) {
				prepared = true;
				await runStepPreparation(step);
			} else {
				updateTargetPosition();
			}

			if (targetRect || attempts >= NAVIGATION_MAX_ATTEMPTS) {
				if (intervalId !== null) {
					clearInterval(intervalId);
				}
			}
		};

		setTimeout(() => {
			tryUpdate();
			intervalId = window.setInterval(tryUpdate, NAVIGATION_RETRY_MS);
		}, NAVIGATION_START_DELAY_MS);
	};

	return {
		// Getters
		get isActive() {
			return isActive;
		},
		get showWelcome() {
			return showWelcome;
		},
		get currentStep() {
			return getCurrentStep();
		},
		get currentStepIndex() {
			return currentStepIndex;
		},
		get totalSteps() {
			return steps.length;
		},
		get targetRect() {
			return targetRect;
		},

		/** Set the navigation function (called by layout) */
		setNavigate(fn: (path: string) => void) {
			navigateFunction = fn;
		},

		/** Show the welcome modal */
		showWelcomeModal() {
			showWelcome = true;
		},

		/** Hide the welcome modal */
		hideWelcome() {
			showWelcome = false;
		},

		/** Start the tutorial */
		start() {
			steps = [...TUTORIAL_STEPS];
			currentStepIndex = 0;
			showWelcome = false;
			isActive = true;

			const firstStep = steps[0];
			if (firstStep?.navigateTo) {
				navigateAndUpdate(firstStep.navigateTo, firstStep);
			} else {
				runStepPreparation(firstStep);
			}
		},

		/** Go to next step */
		next() {
			if (currentStepIndex < steps.length - 1) {
				currentStepIndex++;
				const step = getCurrentStep();

				if (step?.navigateTo) {
					navigateAndUpdate(step.navigateTo, step);
				} else {
					runStepPreparation(step);
				}
			} else {
				// Tutorial complete
				this.complete();
			}
		},

		/** Go to previous step */
		back() {
			if (currentStepIndex > 0) {
				currentStepIndex--;
				const step = getCurrentStep();

				if (step?.navigateTo) {
					navigateAndUpdate(step.navigateTo, step);
				} else {
					runStepPreparation(step);
				}
			}
		},

		/** Skip the tutorial */
		skip() {
			isActive = false;
			showWelcome = false;
			steps = [];
			currentStepIndex = 0;
			targetRect = null;
		},

		/** Complete the tutorial */
		complete() {
			isActive = false;
			showWelcome = false;
			steps = [];
			currentStepIndex = 0;
			targetRect = null;
		},

		/** Update target position (call on resize/scroll) */
		updateTargetPosition,

		/** Check if we're on the last step */
		get isLastStep() {
			return currentStepIndex === steps.length - 1;
		},

		/** Check if we're on the first step */
		get isFirstStep() {
			return currentStepIndex === 0;
		},

		/** Check if current step is a login step */
		get isLoginStep() {
			const step = getCurrentStep();
			return step?.type === 'login';
		},
	};
}

/** Global tutorial store instance */
export const tutorialStore = createTutorialStore();
