// Shared Linkwarden connection check: fetches tags and collections via the plugin backend and
// tracks whether both succeeded. Used by ConnectionSettings.svelte (the indicator + Token/URL
// fields, shared by every action's property inspector) and by simple/+page.svelte (the Add Link
// action, which also needs the fetched tag/collection lists for its dropdowns).
import {derived, get, writable} from "svelte/store";
import {eventTarget, sendToPlugin} from "@openaction/svelte-pi";

export type Tag = {id: number; name: string};
export type Collection = {id: number; name: string};

export const tags = writable<Tag[]>([]);
export const tagsError = writable("");
const tagsFetched = writable(false);

export const collections = writable<Collection[]>([]);
export const collectionsError = writable("");
const collectionsFetched = writable(false);

// "checking" until both requests have returned at least once, then "connected" only if neither
// reported an error - this is what proves the token/instance URL can actually pull real tags and
// collections, not just that they're non-empty strings.
export const connectionStatus = derived(
	[tagsFetched, collectionsFetched, tagsError, collectionsError],
	([$tagsFetched, $collectionsFetched, $tagsError, $collectionsError]) =>
		!$tagsFetched || !$collectionsFetched ? "checking" : $tagsError || $collectionsError ? "error" : "connected",
);

let listening = false;
let connectionEpoch = 0;

// sendToPlugin() silently drops the message if the underlying socket isn't open yet, which it
// often isn't the instant a property inspector page mounts (the socket is still mid-handshake at
// that point) - this is inherent to how @openaction/svelte-pi connects, not something a call site
// can avoid by waiting for a "ready" signal (there isn't one in its public API). So retry a few
// times here instead, purely based on whether a response has arrived.
function requestWithRetry(epoch: number, event: string, isDone: () => boolean, attemptsLeft = 5, delayMs = 300) {
	if (epoch !== connectionEpoch) return;
	sendToPlugin({event});
	if (attemptsLeft <= 1) return;
	setTimeout(() => {
		if (epoch === connectionEpoch && !isDone()) {
			requestWithRetry(epoch, event, isDone, attemptsLeft - 1, delayMs);
		}
	}, delayMs);
}

function ensureListening() {
	if (listening) return;
	listening = true;
	eventTarget.addEventListener("sendToPropertyInspector", ((event: CustomEvent) => {
		const payload = event.detail?.payload;
		if (payload?.event === "tags") {
			tags.set(payload.tags ?? []);
			tagsError.set(payload.error ?? "");
			tagsFetched.set(true);
		} else if (payload?.event === "collections") {
			collections.set(payload.collections ?? []);
			collectionsError.set(payload.error ?? "");
			collectionsFetched.set(true);
		}
	}) as EventListener);
}

export function refreshConnection() {
	ensureListening();
	tagsFetched.set(false);
	collectionsFetched.set(false);
	connectionEpoch++;
	const epoch = connectionEpoch;
	requestWithRetry(epoch, "getTags", () => get(tagsFetched));
	requestWithRetry(epoch, "getCollections", () => get(collectionsFetched));
}