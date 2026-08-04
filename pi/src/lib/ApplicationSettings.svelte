<script lang="ts">
	import {actionSettings, globalSettings} from "$lib/openaction-pi";

	let editing = false;
    let token = "";
    let tokenOverride = "";
    let instanceUrl = "";
    let instanceUrlOverride = "";
    let description = "";

    $: {
        if ($globalSettings.token != undefined) {
            token = $globalSettings.token;
        }
        if ($actionSettings.token != undefined) {
            tokenOverride = $actionSettings.token;
        }
        if ($globalSettings.instanceUrl != undefined) {
            instanceUrl = $globalSettings.instanceUrl;
        }
        if ($actionSettings.instanceUrl != undefined) {
            instanceUrlOverride = $actionSettings.instanceUrl;
        }
        if ($actionSettings.description != undefined) {
            description = $actionSettings.description;
        }
    }

    function handleSave() {
        $globalSettings = {
            ...$globalSettings,
            token,
            instanceUrl,
        };
        $actionSettings = {
            ...$actionSettings,
            token: tokenOverride,
            instanceUrl: instanceUrlOverride,
            description,
        };
        editing = false;
    }

    function handleCancel() {
        token = $globalSettings.token || "";
        tokenOverride = $actionSettings.token || "";
        instanceUrl = $globalSettings.instanceUrl || "";
        instanceUrlOverride = $actionSettings.instanceUrl || "";
        description = $actionSettings.description || "";
        editing = false;
    }

    function maskSecret(secret: string): string {
        return secret ? "•".repeat(secret.length) : "";
    }
</script>

<h2 class="mb-3 text-sm font-semibold text-neutral-100">
    Linkwarden Settings
</h2>

<div class="mb-2 flex items-center gap-2">
    <span class="min-w-22.5 text-xs font-medium text-neutral-200"> API Token: </span>
    {#if editing}
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
    <span class="min-w-22.5 text-xs font-medium text-neutral-200"> Override Token: </span>
    {#if editing}
        <input
                id="tokenOverride"
                type="password"
                bind:value={tokenOverride}
                placeholder="Optional, overrides the global token for this button"
                class="flex-1 rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 placeholder-neutral-500 focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 focus:outline-none"
        />
    {:else}
        <span class="text-xs text-neutral-400">{maskSecret(tokenOverride) || "Not set"}</span>
    {/if}
</div>

<div class="mb-2 flex items-center gap-2">
    <span class="min-w-22.5 text-xs font-medium text-neutral-200"> Instance URL: </span>
    {#if editing}
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

<div class="mb-3 flex items-center gap-2">
    <span class="min-w-22.5 text-xs font-medium text-neutral-200"> Override URL: </span>
    {#if editing}
        <input
                id="instanceUrlOverride"
                type="text"
                bind:value={instanceUrlOverride}
                placeholder="Optional, overrides the global instance URL for this button"
                class="flex-1 rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 placeholder-neutral-500 focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 focus:outline-none"
        />
    {:else}
        <span class="text-xs text-neutral-400">{instanceUrlOverride || "Not set"}</span>
    {/if}
</div>

<div class="mb-3 flex items-center gap-2">
    <span class="min-w-22.5 text-xs font-medium text-neutral-200"> Description: </span>
    {#if editing}
        <textarea
                id="description"
                bind:value={description}
                placeholder="Optional, overrides the default description used for added links"
                rows="2"
                class="flex-1 rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 placeholder-neutral-500 focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 focus:outline-none"
        ></textarea>
    {:else}
        <span class="text-xs text-neutral-400">{description || "Not set"}</span>
    {/if}
</div>

{#if editing}
    <div class="mb-3 flex gap-2">
        <button
                on:click={handleSave}
                class="cursor-pointer rounded bg-neutral-700 px-3 py-1 text-xs text-white hover:bg-neutral-600"
        >
            Save
        </button>
        <button
                on:click={handleCancel}
                class="cursor-pointer rounded bg-neutral-800 px-3 py-1 text-xs text-neutral-300 hover:bg-neutral-700"
        >
            Cancel
        </button>
    </div>
{:else}
    <button
            on:click={() => (editing = true)}
            class="cursor-pointer rounded bg-neutral-700 px-3 py-1 text-xs text-white hover:bg-neutral-600"
    >
        Edit
    </button>
{/if}
