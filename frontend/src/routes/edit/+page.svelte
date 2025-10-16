<script lang="ts">
  import axios from "axios";
  import {showToastOnError, showToastOnErrorP1} from "../../util/errorParser";
  import { onMount } from "svelte";
  import { toast } from "@zerodevx/svelte-toast";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  
  // URL 파라미터에서 장치 ID 가져오기
  let deviceId: string | null = $state(null);
  let isEditMode = $state(false);
  let isSaving = $state(false);
  let req = $state<PostDeviceReq>({
    id: 0, name: "", mac: "", ip: ""
  })


  onMount(() => {
    const deviceId = page.url.searchParams.get("id");
    isEditMode = deviceId != null;
    if (isEditMode) loadDevice(deviceId!);
  });

  // 장치 정보 로딩
  const loadDevice = showToastOnErrorP1(async (deviceId: string) => {
    const res = await axios.get(`/device/${deviceId}`);
    req = res.data;
  });

  const saveDevice = showToastOnError(async () => {
    isSaving = true;
    if (isEditMode) {
      await axios.put(`/device`, req);
      toast.push("Device has been updated.");
    } else {
      await axios.post("/device", req);
      toast.push("Device has been added.");
    }
    isSaving = false;
    goto("/main");
  }, () => (isSaving = false));

  // 취소
  const cancel = () => goto("/main");
</script>

<div class="main-container">
  <div class="main-content">
    <!-- 헤더 섹션 -->
    <div class="header-section">
      <div class="title-area">
        <h1 class="main-title">{isEditMode ? "Edit Device" : "Add Device"}</h1>
        <p class="main-subtitle">
          {isEditMode
            ? "Update device information"
            : "Register a new WOL device"}
        </p>
      </div>
    </div>

      <!-- Form Card -->
      <div class="form-card">
        <form class="common-form" onsubmit={(e) => { e.preventDefault(); saveDevice(); }}>
          <!-- Device Name -->
          <div class="form-group">
            <label for="deviceName">Device Name</label>
            <input
              id="deviceName"
              type="text"
              bind:value={req.name}
              placeholder="Enter device name"
              disabled={isSaving}
              required
            />
          </div>

          <!-- MAC Address -->
          <div class="form-group">
            <label for="deviceMac">MAC Address</label>
            <input
              id="deviceMac"
              type="text"
              bind:value={req.mac}
              placeholder="00:11:22:33:44:55"
              disabled={isSaving}
              required
            />
          </div>
          
          <!-- IP Address -->
          <div class="form-group">
            <label for="ip">IP (optional)</label>
            <input
                id="ip"
                type="text"
                bind:value={req.ip}
                placeholder="192.168.0.1"
                disabled={isSaving}
            />
          </div>

          <!-- Info Box -->
          <div class="info-box">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="16" x2="12" y2="12"></line>
              <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
            <div>
              <strong>MAC Address format:</strong> Enter the MAC address in the
              format <code>XX:XX:XX:XX:XX:XX</code> (e.g., 00:11:22:33:44:55)
            </div>
          </div>

          <!-- Action Buttons -->
          <div class="button-group">
            <button
              type="submit"
              class="action-button save-button"
              disabled={isSaving}
            >
              {#if isSaving}
                <span class="spinner"></span>
              {:else}
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
              {/if}
              {isEditMode ? "Update Device" : "Add Device"}
            </button>
            <button
              type="button"
              class="action-button cancel-button"
              onclick={cancel}
              disabled={isSaving}
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
              Cancel
            </button>
          </div>
        </form>
      </div>
  </div>
</div>

<style lang="scss">
  .main-container {
    min-height: 100vh;
    background: linear-gradient(135deg, #fafafa 0%, #f0f0f0 100%);
    padding: 40px 20px;
    position: relative;

    &::before {
      content: "";
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-image: repeating-linear-gradient(
        0deg,
        transparent,
        transparent 2px,
        rgba(0, 0, 0, 0.015) 2px,
        rgba(0, 0, 0, 0.015) 4px
      );
      pointer-events: none;
    }
  }

  .main-content {
    max-width: 640px;
    margin: 0 auto;
    position: relative;
  }

  /* 헤더 섹션 */
  .header-section {
    margin-bottom: 40px;
  }

  .title-area {
    text-align: center;
  }

  .main-title {
    font-size: 36px;
    font-weight: bold;
    color: var(--color-black);
    margin: 0 0 8px 0;
  }

  .main-subtitle {
    font-size: 16px;
    color: var(--color-dark-gray);
    margin: 0;
    font-weight: normal;
  }

  /* Form Card */
  .form-card {
    background: var(--color-white);
    border-radius: 12px;
    padding: 40px;
    border: 2px solid var(--color-border);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  }

  /* Button Group */
  .button-group {
    display: flex;
    gap: 12px;
    margin-top: 32px;
  }

  .save-button {
    flex: 1;
  }

  .cancel-button {
    flex: 1;
    background: var(--color-dark-gray);

    &:hover:not(:disabled) {
      background: #555555;
    }
  }

  code {
    background: rgba(0, 0, 0, 0.06);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: "Courier New", monospace;
    font-size: 12px;
  }

  /* 반응형 디자인 */
  @media (max-width: 768px) {
    .main-container {
      padding: 24px 16px;
    }

    .main-title {
      font-size: 28px;
    }

    .main-subtitle {
      font-size: 14px;
    }

    .form-card {
      padding: 32px 24px;
    }
  }

  @media (max-width: 480px) {
    .button-group {
      flex-direction: column;
    }

    .form-card {
      padding: 24px 20px;
    }
  }
</style>