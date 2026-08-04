import { get, writable } from "svelte/store"; // Local fork of @openaction/svelte-pi's dist/index.js.

// Local fork of @openaction/svelte-pi's dist/index.js.
//
// The published package sends "context" as the action instance's context
// (parsed out of inActionInfo) on every outgoing event: setSettings,
// sendToPlugin, getGlobalSettings, setGlobalSettings. Under real Stream Deck
// this is rejected for all of them ("wrong context"), confirmed via the
// plugin log: the session registers as one UUID (the property inspector's
// own registered UUID, the second connectElgatoStreamDeckSocket argument),
// but the library then sends a *different* UUID (the action instance
// context) as "context" on every message. OpenDeck doesn't validate this, so
// the bug never showed up there. The fix is to use the property inspector's
// own registered UUID as "context" everywhere instead.th
export const actionSettings = writable<Record<string, any>>({});
export const globalSettings = writable<Record<string, any>>({});
export const eventTarget = new EventTarget();

let ws: WebSocket | undefined;
let action: string;
let propertyInspectorUUID: string;

export function sendToPlugin(payload: unknown) {
	if (ws?.readyState == WebSocket.OPEN) {
		ws.send(
			JSON.stringify({
				event: "sendToPlugin",
				action,
				context: propertyInspectorUUID,
				payload,
			}),
		);
	} else {
		console.warn(
			"Failed to send sendToPlugin event: not connected to OpenAction server",
		);
	}
}

export function openUrl(url: string) {
	if (ws?.readyState == WebSocket.OPEN) {
		ws.send(JSON.stringify({ event: "openUrl", payload: { url } }));
	} else {
		console.warn(
			"Failed to send openUrl event: not connected to OpenAction server",
		);
	}
}

// @ts-expect-error injected by app.html's connect shim
if (globalThis.connectOpenActionSocketData) {
	const [port, uuid, registerEvent, , actionInfo] =
		// @ts-expect-error injected by app.html's connect shim
		await globalThis.connectOpenActionSocketData;
	propertyInspectorUUID = uuid;

	ws = new WebSocket("ws://localhost:" + port);
	const actionData = JSON.parse(actionInfo);
	action = actionData.action;

	let actionSettingsSubscribed = false;
	let globalSettingsSubscribed = false;

	actionSettings.set(actionData.payload.settings ?? {});
	actionSettings.subscribe((settings) => {
		if (!actionSettingsSubscribed) {
			actionSettingsSubscribed = true;
			return;
		}
		ws!.send(
			JSON.stringify({
				event: "setSettings",
				action,
				context: propertyInspectorUUID,
				payload: settings,
			}),
		);
	});
	globalSettings.subscribe((settings) => {
		if (!globalSettingsSubscribed) {
			globalSettingsSubscribed = true;
			return;
		}
		ws!.send(
			JSON.stringify({
				event: "setGlobalSettings",
				context: propertyInspectorUUID,
				payload: settings,
			}),
		);
	});

	ws.onopen = () => {
		ws!.send(
			JSON.stringify({ event: registerEvent, uuid: propertyInspectorUUID }),
		);
		ws!.send(
			JSON.stringify({
				event: "getGlobalSettings",
				context: propertyInspectorUUID,
			}),
		);
	};

	ws.onmessage = (event) => {
		const json = JSON.parse(event.data);
		if (json.event == "didReceiveSettings") {
			const settings = json.payload.settings;
			if (settings != get(actionSettings)) actionSettings.set(settings);
		} else if (json.event == "didReceiveGlobalSettings") {
			const settings = json.payload.settings;
			if (settings != get(globalSettings)) globalSettings.set(settings);
		}
		eventTarget.dispatchEvent(new CustomEvent(json.event, { detail: json }));
	};

	ws.onerror = (event) => {
		console.error("Encountered a WebSocket error:", event);
	};

	ws.onclose = () => {
		console.error("WebSocket connection to OpenAction server closed");
	};
} else {
	console.error(
		"Failed to connect to OpenAction server: connection details not provided",
	);
}
