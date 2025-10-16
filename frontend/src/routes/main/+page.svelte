<script lang="ts">
  import axios from "axios";
  import {showToastOnError} from "../../util/errorParser";
  import {onMount} from "svelte";
  import {toast} from "@zerodevx/svelte-toast";
  import {goto} from "$app/navigation";
  
  let devices: GetDeviceRes[] = $state([]);
  let isLoading = $state(true);

  // 장치 목록 불러오기
  const loadDevices = showToastOnError(async () => {
    isLoading = true;
    const res = await axios.get('/devices');
    devices = res.data;
  }, () => isLoading = false);

  // Delete device
  const deleteDevice = (id: number) => showToastOnError(async () => {
    if (!confirm('Are you sure you want to delete this device?')) return;
    await axios.delete(`/device/${id}`);
    toast.push('Device has been deleted.');
    await loadDevices();
  })();

  // Move device order
  const moveDevice = (id: number, direction: 'up' | 'down') => showToastOnError(async () => {
    await axios.put(`/device/move`, { id,direction });
    await loadDevices();
  })();

  onMount(() => {
    loadDevices();
  });
</script>

<div class="main-container">
  <div class="main-content">
    <!-- 헤더 섹션 -->
    <div class="header-section">
      <div class="title-area">
        <h1 class="main-title">WOL Devices</h1>
        <p class="main-subtitle">Wake-on-LAN Device Management</p>
      </div>
      <button class="add-button action-button" onclick={() => goto('/edit')}>
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        Add Device
      </button>
    </div>

    <!-- Loading State -->
    {#if isLoading}
      <div class="checking-container">
        <span class="spinner large"></span>
        <p class="checking-text">Loading device list...</p>
      </div>
    {:else if devices.length === 0}
      <!-- Empty State -->
      <div class="empty-state">
        <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="7" width="20" height="14" rx="2" ry="2"></rect>
          <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"></path>
        </svg>
        <h3>No devices registered</h3>
        <p>Add a new WOL device to get started.</p>
      </div>
    {:else}
      <!-- Device List -->
      <div class="devices-list">
        {#each devices as device (device.id)}
          <div class="device-card">
            <div class="device-info">
              <div class="device-icon">
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="2" y="7" width="20" height="14" rx="2" ry="2"></rect>
                  <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"></path>
                </svg>
              </div>
              <div class="device-details">
                <h3 class="device-name">{device.name}</h3>
                <p class="device-mac">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="5" y="2" width="14" height="20" rx="2" ry="2"></rect>
                    <line x1="12" y1="18" x2="12.01" y2="18"></line>
                  </svg>
                  {device.mac}
                </p>
              </div>
            </div>
            <div class="device-actions">
              <button
                class="icon-button move-up-button"
                aria-label="Move Up"
                title="Move Up"
                disabled={devices.indexOf(device) === 0}
                onclick={() => moveDevice(device.id, 'up')}>
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="18 15 12 9 6 15"></polyline>
                </svg>
              </button>
              <button
                class="icon-button move-down-button"
                aria-label="Move Down"
                title="Move Down"
                disabled={devices.indexOf(device) === devices.length - 1}
                onclick={() => moveDevice(device.id, 'down')}>
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="6 9 12 15 18 9"></polyline>
                </svg>
              </button>
              <button class="icon-button edit-button" aria-label="Edit" title="Edit"
                      onclick={() => goto(`/edit?id=${device.id}`)}>
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
              </button>
              <button class="icon-button delete-button" aria-label="Delete" title="Delete" onclick={() => deleteDevice(device.id)}>
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="3 6 5 6 21 6"></polyline>
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                  <line x1="10" y1="11" x2="10" y2="17"></line>
                  <line x1="14" y1="11" x2="14" y2="17"></line>
                </svg>
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
  .main-container {
    min-height: 100vh;
    background: linear-gradient(135deg, #fafafa 0%, #f0f0f0 100%);
    padding: 40px 20px;
    position: relative;

    &::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-image: 
        repeating-linear-gradient(0deg,
            transparent, transparent 2px,
            rgba(0, 0, 0, 0.015) 2px,
            rgba(0, 0, 0, 0.015) 4px
        );
      pointer-events: none;
    }
  }

  .main-content {
    max-width: 1200px;
    margin: 0 auto;
    position: relative;
  }

  /* 헤더 섹션 */
  .header-section {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 40px;
    flex-wrap: wrap;
    gap: 20px;
  }

  .title-area {
    flex: 1;
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

  .add-button {
    padding: 12px 24px;
    font-size: 15px;
    white-space: nowrap;
  }

  /* 장치 그리드 */
  .devices-list {
    display: flex;
    flex-direction: column;
    gap: 20px;
    margin-bottom: 40px;
    /* 장치 카드 */
    .device-card {
      background: var(--color-white);
      border-radius: 12px;
      padding: 24px;
      border: 2px solid var(--color-border);
      transition: var(--transition-base);
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 16px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
    }

    .device-info {
      display: flex;
      align-items: center;
      gap: 16px;
      flex: 1;
      min-width: 0;
    }

    .device-icon {
      width: 48px;
      height: 48px;
      background: var(--color-light-gray);
      border-radius: 10px;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      border: 2px solid var(--color-border);
      transition: var(--transition-base);

      svg {
        color: var(--color-black);
      }
    }

    .device-details {
      flex: 1;
      min-width: 0;
    }

    .device-name {
      font-size: 18px;
      font-weight: bold;
      color: var(--color-black);
      margin: 0 0 6px 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .device-mac {
      font-size: 13px;
      color: var(--color-dark-gray);
      margin: 0;
      font-family: 'Courier New', monospace;
      display: flex;
      align-items: center;
      gap: 6px;

      svg {
        flex-shrink: 0;
      }
    }

    .device-actions {
      display: flex;
      gap: 8px;
      flex-shrink: 0;
    }

    .icon-button {
      width: 36px;
      height: 36px;
      border-radius: 8px;
      border: 2px solid var(--color-border);
      background: var(--color-light-gray);
      cursor: pointer;
      transition: var(--transition-base);
      display: flex;
      align-items: center;
      justify-content: center;

      &:hover:not(:disabled) {
        background: var(--color-white);
      }

      &:disabled {
        opacity: 0.4;
        cursor: not-allowed;
      }
    }

    .move-up-button {
      svg {
        color: var(--color-black);
      }
    }

    .move-down-button {
      svg {
        color: var(--color-black);
      }
    }
  }


  /* 빈 상태 */
  .empty-state {
    text-align: center;
    padding: 80px 20px;

    svg {
      color: var(--color-dark-gray);
      opacity: 0.4;
      margin-bottom: 24px;
    }

    h3 {
      font-size: 22px;
      font-weight: 600;
      color: var(--color-black);
      margin: 0 0 12px 0;
    }

    p {
      font-size: 15px;
      color: var(--color-dark-gray);
      margin: 0;
    }
  }

  /* 반응형 디자인 */
  @media (max-width: 768px) {
    .main-container {
      padding: 24px 16px;
    }

    .header-section {
      flex-direction: column;
      align-items: stretch;
      gap: 16px;
    }

    .title-area {
      text-align: center;
    }

    .main-title {
      font-size: 28px;
    }

    .main-subtitle {
      font-size: 14px;
    }

    .add-button {
      width: 100%;
    }

    .devices-list {
      grid-template-columns: 1fr;
      gap: 16px;
    }

    .device-card {
      padding: 20px;
    }
  }

  @media (max-width: 480px) {
    .device-card {
      flex-direction: column;
      align-items: stretch;
    }

    .device-info {
      flex-direction: column;
      align-items: center;
      text-align: center;
    }

    .device-actions {
      justify-content: center;
      width: 100%;
      gap: 12px;

      .icon-button {
        flex: 1;
        max-width: 120px;
      }
    }
  }
</style>