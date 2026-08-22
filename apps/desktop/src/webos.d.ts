interface WebOSServiceResponse {
  returnValue: boolean;
  [key: string]: unknown;
}

interface WebOSServiceRequest {
  cancel(): void;
}

interface WebOSService {
  request(
    uri: string,
    options: {
      method?: string;
      parameters?: Record<string, unknown>;
      onSuccess?: (response: WebOSServiceResponse) => void;
      onFailure?: (response: WebOSServiceResponse) => void;
      onComplete?: (response: WebOSServiceResponse) => void;
      subscribe?: boolean;
    },
  ): WebOSServiceRequest;
}

interface WebOSDeviceInfo {
  modelName: string;
  modelNameAscii: string;
  tvSystemName: string;
  broadcastCountry: string;
  sdkVersion?: string;
  uhd?: boolean;
  uhd8K?: boolean;
  hdr10?: boolean;
  dolbyVision?: boolean;
  dolbyAtmos?: boolean;
  screenWidth?: number;
  screenHeight?: number;
}

interface WebOSAPI {
  service: WebOSService;
  deviceInfo(callback: (info: WebOSDeviceInfo) => void): void;
  fetchAppInfo(callback: (info: Record<string, unknown>) => void, path?: string): void;
  platformBack(): void;
}

interface PalmSystemAPI {
  launchParams: string;
  activated: boolean;
  activate(): void;
  deactivate(): void;
  stagePreparing(): void;
  stageReady(): void;
}

declare var webOS: WebOSAPI | undefined;
declare var PalmSystem: PalmSystemAPI | undefined;

interface HTMLVideoElement {
  setMediaOption?(option: string): void;
  mediaOption?: string;
}
