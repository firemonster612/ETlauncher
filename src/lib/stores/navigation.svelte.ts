import { Layers, Package, Terminal, Settings, Users } from '@lucide/svelte';
import type { Component } from 'svelte';

export interface NavItem {
	id: string;
	label: string;
	href: string;
	icon: Component;
}

export const navItems: NavItem[] = [
	{ id: 'instances', label: 'Instances', href: '/instances', icon: Layers },
	{ id: 'modpacks', label: 'Modpacks', href: '/modpacks', icon: Package },
	{ id: 'console', label: 'Console', href: '/console', icon: Terminal },
	{ id: 'accounts', label: 'Accounts', href: '/accounts', icon: Users },
	{ id: 'settings', label: 'Settings', href: '/settings', icon: Settings },
];
