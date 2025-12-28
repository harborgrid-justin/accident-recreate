import { contextBridge, ipcRenderer, IpcRendererEvent } from 'electron';

export interface ElectronAPI {
  getAppVersion: () => Promise<string>;
  getAppPath: (name: string) => Promise<string>;
  selectFile: (options: Electron.OpenDialogOptions) => Promise<string | null>;
  selectDirectory: () => Promise<string | null>;
  saveFile: (defaultPath: string, data: string) => Promise<boolean>;
  readFile: (filePath: string) => Promise<string | null>;
  showMessageBox: (options: Electron.MessageBoxOptions) => Promise<Electron.MessageBoxReturnValue>;
  getApiUrl: () => Promise<string>;

  onMenuNewCase: (callback: () => void) => () => void;
  onMenuOpenCase: (callback: () => void) => () => void;
  onMenuSaveCase: (callback: () => void) => () => void;
  onMenuExportPdf: (callback: () => void) => () => void;
  onMenuRunSimulation: (callback: () => void) => () => void;
  onMenuResetSimulation: (callback: () => void) => () => void;
  onMenuViewResults: (callback: () => void) => () => void;
  onMenuShowDocumentation: (callback: () => void) => () => void;
}

const ALLOWED_CHANNELS = {
  invoke: [
    'get-app-version',
    'get-app-path',
    'select-file',
    'select-directory',
    'save-file',
    'read-file',
    'show-message-box',
    'get-api-url',
  ],
  on: [
    'menu-new-case',
    'menu-open-case',
    'menu-save-case',
    'menu-export-pdf',
    'menu-run-simulation',
    'menu-reset-simulation',
    'menu-view-results',
    'menu-show-documentation',
  ],
};

function isAllowedChannel(channel: string, type: 'invoke' | 'on'): boolean {
  return ALLOWED_CHANNELS[type].includes(channel);
}

function createMenuEventHandler(channel: string) {
  return (callback: () => void): (() => void) => {
    const subscription = (_event: IpcRendererEvent) => {
      callback();
    };

    ipcRenderer.on(channel, subscription);

    return () => {
      ipcRenderer.removeListener(channel, subscription);
    };
  };
}

const electronAPI: ElectronAPI = {
  getAppVersion: (): Promise<string> => {
    if (!isAllowedChannel('get-app-version', 'invoke')) {
      return Promise.reject(new Error('Channel not allowed'));
    }
    return ipcRenderer.invoke('get-app-version');
  },

  getAppPath: (name: string): Promise<string> => {
    if (!isAllowedChannel('get-app-path', 'invoke')) {
      return Promise.reject(new Error('Channel not allowed'));
    }
    return ipcRenderer.invoke('get-app-path', name);
  },

  selectFile: (options: Electron.OpenDialogOptions): Promise<string | null> => {
    if (!isAllowedChannel('select-file', 'invoke')) {
      return Promise.reject(new Error('Channel not allowed'));
    }
    return ipcRenderer.invoke('select-file', options);
  },

  selectDirectory: (): Promise<string | null> => {
    if (!isAllowedChannel('select-directory', 'invoke')) {
      return Promise.reject(new Error('Channel not allowed'));
    }
    return ipcRenderer.invoke('select-directory');
  },

  saveFile: (defaultPath: string, data: string): Promise<boolean> => {
    if (!isAllowedChannel('save-file', 'invoke')) {
      return Promise.reject(new Error('Channel not allowed'));
    }
    return ipcRenderer.invoke('save-file', defaultPath, data);
  },

  readFile: (filePath: string): Promise<string | null> => {
    if (!isAllowedChannel('read-file', 'invoke')) {
      return Promise.reject(new Error('Channel not allowed'));
    }
    return ipcRenderer.invoke('read-file', filePath);
  },

  showMessageBox: (options: Electron.MessageBoxOptions): Promise<Electron.MessageBoxReturnValue> => {
    if (!isAllowedChannel('show-message-box', 'invoke')) {
      return Promise.reject(new Error('Channel not allowed'));
    }
    return ipcRenderer.invoke('show-message-box', options);
  },

  getApiUrl: (): Promise<string> => {
    if (!isAllowedChannel('get-api-url', 'invoke')) {
      return Promise.reject(new Error('Channel not allowed'));
    }
    return ipcRenderer.invoke('get-api-url');
  },

  onMenuNewCase: createMenuEventHandler('menu-new-case'),
  onMenuOpenCase: createMenuEventHandler('menu-open-case'),
  onMenuSaveCase: createMenuEventHandler('menu-save-case'),
  onMenuExportPdf: createMenuEventHandler('menu-export-pdf'),
  onMenuRunSimulation: createMenuEventHandler('menu-run-simulation'),
  onMenuResetSimulation: createMenuEventHandler('menu-reset-simulation'),
  onMenuViewResults: createMenuEventHandler('menu-view-results'),
  onMenuShowDocumentation: createMenuEventHandler('menu-show-documentation'),
};

try {
  contextBridge.exposeInMainWorld('electronAPI', electronAPI);
} catch (error) {
  console.error('Failed to expose Electron API:', error);
}

declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}
