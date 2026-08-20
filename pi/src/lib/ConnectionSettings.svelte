<script lang="ts">
	import {onMount} from "svelte";
	import {actionSettings, globalSettings} from "@openaction/svelte-pi";
	import {connectionStatus, tagsError, collectionsError, refreshConnection} from "$lib/connection";

	let editingConnection = false;

    let token = "";
    let instanceUrl = "";

    // There's only ever one *effective* connection per action instance - credentials() on the
    // backend resolves to the override if either is set, otherwise the global settings, never
    // both. So this reuses the single indicator below rather than showing a separate one for
    // "the override", and just labels which source it's currently reporting on.
    $: usingOverride = !!($actionSettings.token || $actionSettings.instanceUrl);

    $: {
        if ($globalSettings.token != undefined) {
            token = $globalSettings.token;
        }
        if ($globalSettings.instanceUrl != undefined) {
            instanceUrl = $globalSettings.instanceUrl;
        }
    }

    onMount(() => {
        refreshConnection();
    });

    function handleSaveConnection() {
        $globalSettings = {
            ...$globalSettings,
            token,
            instanceUrl,
        };
        editingConnection = false;
        refreshConnection();
    }

    function handleCancelConnection() {
        token = $globalSettings.token || "";
        instanceUrl = $globalSettings.instanceUrl || "";
        editingConnection = false;
    }

    function maskSecret(secret: string): string {
        return secret ? "•".repeat(secret.length) : "";
    }
</script>

<h2 class="mb-3 text-sm font-semibold text-neutral-100">
    Linkwarden Connection
</h2>

<div class="mb-1 flex items-center gap-2">
    {#if $connectionStatus === "checking"}
        <span class="h-2 w-2 rounded-full bg-neutral-500"></span>
        <span class="text-xs text-neutral-400">Checking connection...</span>
    {:else if $connectionStatus === "connected"}
        <span class="h-2 w-2 rounded-full bg-green-500"></span>
        <span class="text-xs text-neutral-400">Connected - the instance is reachable</span>
    {:else}
        <span class="h-2 w-2 rounded-full bg-red-500"></span>
        <span class="text-xs text-red-400">Connection failed: {$tagsError || $collectionsError}</span>
    {/if}
</div>
<p class="mb-3 text-xs text-neutral-500">
    {usingOverride ? "Using this button's override settings" : "Using the global connection settings"}
</p>

<div class="mb-2 flex items-center gap-2">
    <span class="min-w-22.5 text-xs font-medium text-neutral-200"> API Token: </span>
    {#if editingConnection}
        <input
                id="token"
                type="password"
                bind:value={token}
                placeholder="Enter the API token from Linkwarden"
                class="flex-1 rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 placeholder-neutral-500 focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 focus:outline-none"
        />
    {:else}
        <span class="text-xs text-neutral-400">{maskSecret(token) || "Not set"}</span>
    {/if}
</div>

<div class="mb-3 flex items-center gap-2">
    <span class="min-w-22.5 text-xs font-medium text-neutral-200"> Instance URL: </span>
    {#if editingConnection}
        <input
                id="instanceUrl"
                type="text"
                bind:value={instanceUrl}
                placeholder="e.g. https://linkwarden.example.com"
                class="flex-1 rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 placeholder-neutral-500 focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 focus:outline-none"
        />
    {:else}
        <span class="text-xs text-neutral-400">{instanceUrl || "Not set"}</span>
    {/if}
</div>

{#if editingConnection}
    <div class="mb-4 flex gap-2">
        <button
                on:click={handleSaveConnection}
                class="cursor-pointer rounded bg-neutral-700 px-3 py-1 text-xs text-white hover:bg-neutral-600"
        >
            Save
        </button>
        <button
                on:click={handleCancelConnection}
                class="cursor-pointer rounded bg-neutral-800 px-3 py-1 text-xs text-neutral-300 hover:bg-neutral-700"
        >
            Cancel
        </button>
    </div>
{:else}
    <button
            on:click={() => (editingConnection = true)}
            class="mb-4 cursor-pointer rounded bg-neutral-700 px-3 py-1 text-xs text-white hover:bg-neutral-600"
    >
        Edit
    </button>
{/if}