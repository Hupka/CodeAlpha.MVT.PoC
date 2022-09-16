import type { Writable } from 'svelte/store';

export type ThemeName = 'light' | 'dark';

export const colorNames = [
	'primary_gradient',
	'secondary',
	'inactive',
	'contrast',
	'contrastsecondary',
	'background',
	'backgroundsecondary'
] as const;

export type ColorName = typeof colorNames[number];
export interface Theme {
	colors: {
		[key in ColorName]: string;
	};
	name: ThemeName;
}
export interface ThemeContextType {
	setTheme: (theme: ThemeName) => void;
	theme: Writable<Theme>;
}

export const themes: { [name in ThemeName]: Theme } = {
	light: {
		colors: {
			primary_gradient: 'linear-gradient(225deg, #0b9cda 0%, #054b8b 100%)',

			background: '#ffffff',
			backgroundsecondary: '#e5e5e5',

			secondary: '#a3a3a3',

			contrast: '#000000',
			contrastsecondary: '#737373',

			inactive: '#bbbbbb80'
		},
		name: 'light'
	},
	dark: {
		colors: {
			primary_gradient: 'linear-gradient(219.24deg, #ff9c64 1.87%, #f84545 68.89%)',
			background: '#262626',
			backgroundsecondary: '#404040',

			secondary: '#a3a3a3',

			contrast: '#F5F5F5',
			contrastsecondary: '#e5e5e5',

			inactive: '#80808080'
		},
		name: 'dark'
	}
};

export function mapColorNameToCssVarString(colorName: ColorName) {
	return `--theme-${colorName}`;
}

export const colors = colorNames.reduce(
	(curr, colorName) => ({ ...curr, [colorName]: `var(${mapColorNameToCssVarString(colorName)})` }),
	{} as { [colorName in ColorName]: string }
);
