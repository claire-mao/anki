<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import type { Preferences } from "@generated/anki/config_pb";
    import { Preferences_Scheduling_NewReviewMix } from "@generated/anki/config_pb";
    import { bridgeCommand, bridgeCommandsAvailable } from "@tslib/bridgecommand";
    import { scheduleGreAtlasAutoSync, syncGreAtlasPractice } from "../gre-sync";

    import GrePageHeader from "../GrePageHeader.svelte";
    import { greDeckOptionsAction, runGreNavAction } from "../gre-navigation";
    import { formatPercent } from "../score-format";
    import { fsrsStatus } from "../summary-metrics";
    import GreButton from "../ui/GreButton.svelte";
    import GreButtonRow from "../ui/GreButtonRow.svelte";
    import GreMetricRow from "../ui/GreMetricRow.svelte";
    import GreText from "../ui/GreText.svelte";
    import SettingsNumber from "./SettingsNumber.svelte";
    import SettingsSection from "./SettingsSection.svelte";
    import SettingsSelect from "./SettingsSelect.svelte";
    import SettingsText from "./SettingsText.svelte";
    import SettingsToggle from "./SettingsToggle.svelte";
    import { queuePreferencesSave } from "./settings-preferences";
    import type { PageData } from "./$types";

    import "../gre.scss";
    import "./settings.scss";

    export let data: PageData;

    const preferences: Preferences = data.preferences;

    $: memory = data.scores.memory!;
    $: deckConfigs = data.deckConfigs;
    $: currentDeckConfig = deckConfigs?.currentDeck?.configId;
    $: activePreset = deckConfigs?.allConfig.find(
        (entry) => entry.config?.id === currentDeckConfig,
    )?.config?.config;
    $: fsrsLabel = fsrsStatus(memory.abstentionRequirements);
    $: canOpenDeckOptions = data.studyStatus.deckExists && data.deckId !== null;

    let rollover = preferences.scheduling?.rollover ?? 4;
    let learnAheadMins = Math.round(
        (preferences.scheduling?.learnAheadSecs ?? 1200) / 60,
    );
    let timeLimitMins = Math.round((preferences.reviewing?.timeLimitSecs ?? 0) / 60);
    let dailyBackups = preferences.backups?.daily ?? 4;
    let weeklyBackups = preferences.backups?.weekly ?? 0;
    let monthlyBackups = preferences.backups?.monthly ?? 0;
    let backupIntervalMins = preferences.backups?.minimumIntervalMins ?? 30;
    let showPlayButtons = !preferences.reviewing!.hideAudioPlayButtons;
    let addingDefaultsToCurrentDeck = !preferences.editing!.addingDefaultsToCurrentDeck;
    let defaultSearchText = preferences.editing!.defaultSearchText;
    let newReviewMix = String(
        preferences.scheduling?.newReviewMix ??
            Preferences_Scheduling_NewReviewMix.DISTRIBUTE,
    );

    const newReviewMixOptions = [
        {
            value: String(Preferences_Scheduling_NewReviewMix.DISTRIBUTE),
            label: "Mix new and review cards",
        },
        {
            value: String(Preferences_Scheduling_NewReviewMix.REVIEWS_FIRST),
            label: "Show review cards first",
        },
        {
            value: String(Preferences_Scheduling_NewReviewMix.NEW_FIRST),
            label: "Show new cards first",
        },
    ];

    function persist(): void {
        queuePreferencesSave(preferences);
    }

    function saveScheduling(): void {
        if (!preferences.scheduling) {
            return;
        }
        preferences.scheduling.rollover = rollover;
        preferences.scheduling.learnAheadSecs = learnAheadMins * 60;
        persist();
    }

    function saveAdvancedScheduling(): void {
        if (!preferences.scheduling) {
            return;
        }
        preferences.scheduling.newReviewMix = Number(
            newReviewMix,
        ) as Preferences_Scheduling_NewReviewMix;
        persist();
    }

    function updateScheduling<K extends keyof NonNullable<Preferences["scheduling"]>>(
        field: K,
        value: NonNullable<Preferences["scheduling"]>[K],
    ): void {
        if (!preferences.scheduling) {
            return;
        }
        preferences.scheduling[field] = value;
        persist();
    }

    function saveTimeLimit(): void {
        if (!preferences.reviewing) {
            return;
        }
        preferences.reviewing.timeLimitSecs = timeLimitMins * 60;
        persist();
    }

    function saveBackups(): void {
        if (!preferences.backups) {
            return;
        }
        preferences.backups.daily = dailyBackups;
        preferences.backups.weekly = weeklyBackups;
        preferences.backups.monthly = monthlyBackups;
        preferences.backups.minimumIntervalMins = backupIntervalMins;
        persist();
    }

    function updateReviewing<K extends keyof NonNullable<Preferences["reviewing"]>>(
        field: K,
        value: NonNullable<Preferences["reviewing"]>[K],
    ): void {
        if (!preferences.reviewing) {
            return;
        }
        preferences.reviewing[field] = value;
        persist();
    }

    function updateEditing<K extends keyof NonNullable<Preferences["editing"]>>(
        field: K,
        value: NonNullable<Preferences["editing"]>[K],
    ): void {
        if (!preferences.editing) {
            return;
        }
        preferences.editing[field] = value;
        persist();
    }

    function runBridge(command: string): void {
        if (bridgeCommandsAvailable()) {
            bridgeCommand(command);
        }
    }
</script>

<GrePageHeader
    title="Settings"
    icon="settings"
    subtitle="Review rhythm, practice sessions, and sync."
/>

<div class="settings-page">
    <SettingsSection
        title="Study"
        description="Review rhythm and what you see during flashcard sessions."
    >
        <SettingsNumber
            label="Next day starts at"
            description="When a new study day begins."
            bind:value={rollover}
            min={0}
            max={23}
            suffix="hours past midnight"
            on:change={saveScheduling}
        />
        <SettingsNumber
            label="Learn ahead limit"
            description="How far ahead learning cards can be shown."
            bind:value={learnAheadMins}
            min={0}
            max={999}
            suffix="minutes"
            on:change={saveScheduling}
        />
        <SettingsToggle
            label="Show remaining card count"
            bind:checked={preferences.reviewing!.showRemainingDueCounts}
            on:change={() =>
                updateReviewing(
                    "showRemainingDueCounts",
                    preferences.reviewing!.showRemainingDueCounts,
                )}
        />
        <SettingsToggle
            label="Show next review on answer buttons"
            bind:checked={preferences.reviewing!.showIntervalsOnButtons}
            on:change={() =>
                updateReviewing(
                    "showIntervalsOnButtons",
                    preferences.reviewing!.showIntervalsOnButtons,
                )}
        />
        <SettingsToggle
            label="Show play buttons on cards with audio"
            bind:checked={showPlayButtons}
            on:change={() => updateReviewing("hideAudioPlayButtons", !showPlayButtons)}
        />
        <SettingsToggle
            label="Interrupt audio when answering"
            bind:checked={preferences.reviewing!.interruptAudioWhenAnswering}
            on:change={() =>
                updateReviewing(
                    "interruptAudioWhenAnswering",
                    preferences.reviewing!.interruptAudioWhenAnswering,
                )}
        />
    </SettingsSection>

    <SettingsSection title="Practice" description="Time limits for practice sessions.">
        <SettingsNumber
            label="Session time limit"
            description="Optional cap per session. Set to 0 for no limit."
            bind:value={timeLimitMins}
            min={0}
            max={9999}
            suffix="minutes"
            on:change={saveTimeLimit}
        />
    </SettingsSection>

    <SettingsSection title="Practice sync" description="Cross-device GRE Atlas practice data.">
        <GreText variant="body">
            Syncs practice attempts, sessions, generated questions, and calibration
            history via your Anki sync server. This requires a self-hosted sync server
            with GRE Atlas routes enabled; AnkiWeb sign-in alone is not enough. Set a
            custom sync server under Account → Sync &amp; account details, sign in,
            then sync here.
        </GreText>
        <GreButtonRow className="settings-actions">
            <GreButton variant="primary" on:click={() => syncGreAtlasPractice()}>
                Sync practice data now
            </GreButton>
        </GreButtonRow>
    </SettingsSection>

    <SettingsSection title="Account" description="Cloud sync and sign-in.">
        <GreText variant="body">
            Sign in to sync your GRE Atlas collection across devices.
        </GreText>
        <GreButtonRow className="settings-actions">
            <GreButton variant="primary" on:click={() => runBridge("greSyncLogin")}>
                Sign in
            </GreButton>
            <GreButton on:click={() => runBridge("greSyncLogout")}>Sign out</GreButton>
            <GreButton on:click={() => runBridge("greOpenAnkiPreferences")}>
                Sync & account details
            </GreButton>
        </GreButtonRow>
    </SettingsSection>

    <details class="gre-ds-panel settings-advanced">
        <summary class="settings-advanced-summary">
            <span class="settings-advanced-title">Advanced</span>
            <span class="settings-advanced-hint">
                Prediction, scheduling, editing, and backups
            </span>
        </summary>

        <div class="settings-advanced-body">
            <SettingsSection
                title="Prediction"
                description="Deck options that power GRE Atlas score predictions."
            >
                <GreMetricRow label="FSRS" value={fsrsLabel} />
                {#if activePreset}
                    <GreMetricRow
                        label="Desired retention"
                        value={formatPercent(
                            Math.round((activePreset.desiredRetention ?? 0.9) * 100),
                        )}
                    />
                    <GreMetricRow
                        label="Daily new cards"
                        value={String(activePreset.newPerDay)}
                    />
                    <GreMetricRow
                        label="Daily review limit"
                        value={String(activePreset.reviewsPerDay)}
                    />
                {:else}
                    <GreText variant="caption" muted>
                        Open Study once to load built-in GRE flashcards, then return
                        here to configure prediction settings.
                    </GreText>
                {/if}
                {#if canOpenDeckOptions}
                    <GreButtonRow className="settings-actions">
                        <GreButton
                            variant="primary"
                            on:click={() => runGreNavAction(greDeckOptionsAction())}
                        >
                            Open GRE deck options
                        </GreButton>
                    </GreButtonRow>
                {/if}
            </SettingsSection>

            <SettingsSection title="Scheduling">
                <SettingsSelect
                    label="New/review card order"
                    description="How new cards are mixed with reviews during a session."
                    bind:value={newReviewMix}
                    options={newReviewMixOptions}
                    on:change={saveAdvancedScheduling}
                />
                <SettingsToggle
                    label="Use new timezone handling"
                    bind:checked={preferences.scheduling!.newTimezone}
                    on:change={() =>
                        updateScheduling(
                            "newTimezone",
                            preferences.scheduling!.newTimezone,
                        )}
                />
                <SettingsToggle
                    label="Show learning cards before reviews"
                    bind:checked={preferences.scheduling!.dayLearnFirst}
                    on:change={() =>
                        updateScheduling(
                            "dayLearnFirst",
                            preferences.scheduling!.dayLearnFirst,
                        )}
                />
                <SettingsToggle
                    label="Enable load balancer"
                    description="Spread reviews across days to smooth workload."
                    bind:checked={preferences.reviewing!.loadBalancerEnabled}
                    on:change={() =>
                        updateReviewing(
                            "loadBalancerEnabled",
                            preferences.reviewing!.loadBalancerEnabled,
                        )}
                />
                <SettingsToggle
                    label="Enable short-term steps with FSRS"
                    description="Allow learning steps for cards in the FSRS short-term state."
                    bind:checked={preferences.reviewing!.fsrsShortTermWithStepsEnabled}
                    on:change={() =>
                        updateReviewing(
                            "fsrsShortTermWithStepsEnabled",
                            preferences.reviewing!.fsrsShortTermWithStepsEnabled,
                        )}
                />
            </SettingsSection>

            <SettingsSection title="Editing">
                <SettingsToggle
                    label="Paste clipboard images as PNG"
                    bind:checked={preferences.editing!.pasteImagesAsPng}
                    on:change={() =>
                        updateEditing(
                            "pasteImagesAsPng",
                            preferences.editing!.pasteImagesAsPng,
                        )}
                />
                <SettingsToggle
                    label="Paste without Shift strips formatting"
                    bind:checked={preferences.editing!.pasteStripsFormatting}
                    on:change={() =>
                        updateEditing(
                            "pasteStripsFormatting",
                            preferences.editing!.pasteStripsFormatting,
                        )}
                />
                <SettingsToggle
                    label="Generate LaTeX images automatically"
                    bind:checked={preferences.editing!.renderLatex}
                    on:change={() =>
                        updateEditing("renderLatex", preferences.editing!.renderLatex)}
                />
                <SettingsToggle
                    label="Ignore accents in search"
                    bind:checked={preferences.editing!.ignoreAccentsInSearch}
                    on:change={() =>
                        updateEditing(
                            "ignoreAccentsInSearch",
                            preferences.editing!.ignoreAccentsInSearch,
                        )}
                />
                <SettingsToggle
                    label="Adding cards defaults to current deck"
                    bind:checked={addingDefaultsToCurrentDeck}
                    on:change={() =>
                        updateEditing(
                            "addingDefaultsToCurrentDeck",
                            !addingDefaultsToCurrentDeck,
                        )}
                />
                <SettingsText
                    label="Default search text"
                    bind:value={defaultSearchText}
                    on:change={(event) => {
                        defaultSearchText = event.detail;
                        updateEditing("defaultSearchText", defaultSearchText);
                    }}
                />
            </SettingsSection>

            <SettingsSection title="Backups">
                <SettingsNumber
                    label="Daily backups to keep"
                    bind:value={dailyBackups}
                    min={0}
                    max={999}
                    on:change={saveBackups}
                />
                <SettingsNumber
                    label="Weekly backups to keep"
                    bind:value={weeklyBackups}
                    min={0}
                    max={999}
                    on:change={saveBackups}
                />
                <SettingsNumber
                    label="Monthly backups to keep"
                    bind:value={monthlyBackups}
                    min={0}
                    max={999}
                    on:change={saveBackups}
                />
                <SettingsNumber
                    label="Minimum minutes between backups"
                    bind:value={backupIntervalMins}
                    min={0}
                    max={9999}
                    suffix="minutes"
                    on:change={saveBackups}
                />
            </SettingsSection>

            <SettingsSection title="More options">
                <GreText variant="caption" muted>
                    Appearance, language, answer keys, updates, and other app-wide
                    settings open in a separate preferences window.
                </GreText>
                <GreButtonRow className="settings-actions">
                    <GreButton on:click={() => runBridge("greOpenAnkiPreferences")}>
                        Open more preferences…
                    </GreButton>
                </GreButtonRow>
            </SettingsSection>
        </div>
    </details>
</div>
