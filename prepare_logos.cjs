/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

const fs = require('fs');
const path = require('path');

const resourcesDir = path.join(__dirname, 'resources');
const editorMediaDir = path.join(__dirname, 'src/vs/workbench/browser/parts/editor/media');
const browserMediaDir = path.join(__dirname, 'src/vs/workbench/browser/media');

const cleanBase64 = (str) => str.replace(/\n/g, '');

function createSVG(pngPath, opacity = 1.0) {
	const pngData = fs.readFileSync(pngPath);
	const base64 = pngData.toString('base64');
	// Assuming square images based on 'sips' output earlier (1024x1024)
	return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" opacity="${opacity}">
<image href="data:image/png;base64,${cleanBase64(base64)}" width="1024" height="1024"/>
</svg>`;
}

const lightPng = path.join(resourcesDir, 'ride-light.png');
const darkPng = path.join(resourcesDir, 'ride-dark.png');

// 1. Generate Watermarks (Letterpress) - Lower Opacity
const watermarkOpacity = 0.3; // Subtle watermark
const lightWatermark = createSVG(lightPng, watermarkOpacity);
const darkWatermark = createSVG(darkPng, watermarkOpacity);

fs.writeFileSync(path.join(editorMediaDir, 'letterpress-light.svg'), lightWatermark);
fs.writeFileSync(path.join(editorMediaDir, 'letterpress-hcLight.svg'), lightWatermark);
fs.writeFileSync(path.join(editorMediaDir, 'letterpress-dark.svg'), darkWatermark);
fs.writeFileSync(path.join(editorMediaDir, 'letterpress-hcDark.svg'), darkWatermark);

console.log('Updated letterpress SVGs.');

// 2. Generate Icons (Code Icon) - Full Opacity
const iconOpacity = 1.0;
const lightIcon = createSVG(lightPng, iconOpacity);
const darkIcon = createSVG(darkPng, iconOpacity);

// Write to browser/media for usage in CSS
fs.writeFileSync(path.join(browserMediaDir, 'ride-light.svg'), lightIcon);
fs.writeFileSync(path.join(browserMediaDir, 'ride-dark.svg'), darkIcon);

// Overwrite default code-icon.svg with dark (default)
fs.writeFileSync(path.join(browserMediaDir, 'code-icon.svg'), darkIcon);

console.log('Created ride-light.svg, ride-dark.svg and updated code-icon.svg.');
