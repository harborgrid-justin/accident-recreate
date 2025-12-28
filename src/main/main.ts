import { app, BrowserWindow, ipcMain, Menu, dialog } from 'electron';
import * as path from 'path';
import * as fs from 'fs';
import { spawn, ChildProcess } from 'child_process';

class AccuSceneApplication {
  private mainWindow: BrowserWindow | null = null;
  private apiServer: ChildProcess | null = null;
  private readonly isDevelopment: boolean = process.env.NODE_ENV === 'development';
  private readonly apiPort: number = 3001;

  constructor() {
    this.initializeApp();
  }

  private initializeApp(): void {
    app.on('ready', () => this.onReady());
    app.on('window-all-closed', () => this.onWindowAllClosed());
    app.on('activate', () => this.onActivate());
    app.on('before-quit', () => this.onBeforeQuit());
  }

  private async onReady(): Promise<void> {
    try {
      await this.startApiServer();
      await this.createMainWindow();
      this.setupIpcHandlers();
      this.createApplicationMenu();
    } catch (error) {
      console.error('Failed to initialize application:', error);
      app.quit();
    }
  }

  private async createMainWindow(): Promise<void> {
    this.mainWindow = new BrowserWindow({
      width: 1600,
      height: 1000,
      minWidth: 1024,
      minHeight: 768,
      title: 'AccuScene Enterprise',
      backgroundColor: '#1e1e1e',
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
        sandbox: true,
        preload: path.join(__dirname, 'preload.js'),
        devTools: this.isDevelopment,
        webSecurity: true,
      },
      show: false,
    });

    this.mainWindow.once('ready-to-show', () => {
      this.mainWindow?.show();
    });

    this.mainWindow.on('closed', () => {
      this.mainWindow = null;
    });

    if (this.isDevelopment) {
      await this.mainWindow.loadURL('http://localhost:8080');
      this.mainWindow.webContents.openDevTools();
    } else {
      await this.mainWindow.loadFile(path.join(__dirname, '../renderer/index.html'));
    }
  }

  private async startApiServer(): Promise<void> {
    return new Promise((resolve, reject) => {
      const apiPath = this.isDevelopment
        ? path.join(__dirname, '../../src/api/server.ts')
        : path.join(__dirname, '../api/server.js');

      const nodeArgs = this.isDevelopment
        ? ['-r', 'ts-node/register', apiPath]
        : [apiPath];

      this.apiServer = spawn('node', nodeArgs, {
        env: {
          ...process.env,
          PORT: String(this.apiPort),
          NODE_ENV: process.env.NODE_ENV || 'production',
        },
        stdio: 'pipe',
      });

      this.apiServer.stdout?.on('data', (data: Buffer) => {
        console.log(`[API Server]: ${data.toString()}`);
      });

      this.apiServer.stderr?.on('data', (data: Buffer) => {
        console.error(`[API Server Error]: ${data.toString()}`);
      });

      this.apiServer.on('error', (error: Error) => {
        console.error('Failed to start API server:', error);
        reject(error);
      });

      this.apiServer.on('exit', (code: number | null) => {
        console.log(`API server exited with code ${code}`);
      });

      setTimeout(() => resolve(), 3000);
    });
  }

  private setupIpcHandlers(): void {
    ipcMain.handle('get-app-version', () => {
      return app.getVersion();
    });

    ipcMain.handle('get-app-path', (_, name: string) => {
      return app.getPath(name as any);
    });

    ipcMain.handle('select-file', async (_, options: Electron.OpenDialogOptions) => {
      if (!this.mainWindow) return null;
      const result = await dialog.showOpenDialog(this.mainWindow, options);
      return result.filePaths[0] || null;
    });

    ipcMain.handle('select-directory', async () => {
      if (!this.mainWindow) return null;
      const result = await dialog.showOpenDialog(this.mainWindow, {
        properties: ['openDirectory', 'createDirectory'],
      });
      return result.filePaths[0] || null;
    });

    ipcMain.handle('save-file', async (_, defaultPath: string, data: string) => {
      if (!this.mainWindow) return false;
      const result = await dialog.showSaveDialog(this.mainWindow, {
        defaultPath,
        filters: [
          { name: 'JSON Files', extensions: ['json'] },
          { name: 'PDF Files', extensions: ['pdf'] },
          { name: 'All Files', extensions: ['*'] },
        ],
      });

      if (result.canceled || !result.filePath) return false;

      try {
        await fs.promises.writeFile(result.filePath, data, 'utf-8');
        return true;
      } catch (error) {
        console.error('Failed to save file:', error);
        return false;
      }
    });

    ipcMain.handle('read-file', async (_, filePath: string) => {
      try {
        const data = await fs.promises.readFile(filePath, 'utf-8');
        return data;
      } catch (error) {
        console.error('Failed to read file:', error);
        return null;
      }
    });

    ipcMain.handle('show-message-box', async (_, options: Electron.MessageBoxOptions) => {
      if (!this.mainWindow) return null;
      const result = await dialog.showMessageBox(this.mainWindow, options);
      return result;
    });

    ipcMain.handle('get-api-url', () => {
      return `http://localhost:${this.apiPort}`;
    });
  }

  private createApplicationMenu(): void {
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'File',
        submenu: [
          {
            label: 'New Case',
            accelerator: 'CmdOrCtrl+N',
            click: () => {
              this.mainWindow?.webContents.send('menu-new-case');
            },
          },
          {
            label: 'Open Case',
            accelerator: 'CmdOrCtrl+O',
            click: () => {
              this.mainWindow?.webContents.send('menu-open-case');
            },
          },
          {
            label: 'Save Case',
            accelerator: 'CmdOrCtrl+S',
            click: () => {
              this.mainWindow?.webContents.send('menu-save-case');
            },
          },
          { type: 'separator' },
          {
            label: 'Export PDF',
            accelerator: 'CmdOrCtrl+E',
            click: () => {
              this.mainWindow?.webContents.send('menu-export-pdf');
            },
          },
          { type: 'separator' },
          {
            label: 'Exit',
            accelerator: 'CmdOrCtrl+Q',
            click: () => {
              app.quit();
            },
          },
        ],
      },
      {
        label: 'Edit',
        submenu: [
          { role: 'undo' },
          { role: 'redo' },
          { type: 'separator' },
          { role: 'cut' },
          { role: 'copy' },
          { role: 'paste' },
          { role: 'delete' },
          { type: 'separator' },
          { role: 'selectAll' },
        ],
      },
      {
        label: 'View',
        submenu: [
          { role: 'reload' },
          { role: 'forceReload' },
          { type: 'separator' },
          { role: 'resetZoom' },
          { role: 'zoomIn' },
          { role: 'zoomOut' },
          { type: 'separator' },
          { role: 'togglefullscreen' },
        ],
      },
      {
        label: 'Simulation',
        submenu: [
          {
            label: 'Run Simulation',
            accelerator: 'CmdOrCtrl+R',
            click: () => {
              this.mainWindow?.webContents.send('menu-run-simulation');
            },
          },
          {
            label: 'Reset Simulation',
            accelerator: 'CmdOrCtrl+Shift+R',
            click: () => {
              this.mainWindow?.webContents.send('menu-reset-simulation');
            },
          },
          { type: 'separator' },
          {
            label: 'View Results',
            click: () => {
              this.mainWindow?.webContents.send('menu-view-results');
            },
          },
        ],
      },
      {
        label: 'Help',
        submenu: [
          {
            label: 'Documentation',
            click: () => {
              this.mainWindow?.webContents.send('menu-show-documentation');
            },
          },
          {
            label: 'About AccuScene',
            click: () => {
              this.showAboutDialog();
            },
          },
          ...(this.isDevelopment
            ? [
                { type: 'separator' as const },
                { role: 'toggleDevTools' as const },
              ]
            : []),
        ],
      },
    ];

    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
  }

  private showAboutDialog(): void {
    if (!this.mainWindow) return;

    dialog.showMessageBox(this.mainWindow, {
      type: 'info',
      title: 'About AccuScene Enterprise',
      message: 'AccuScene Enterprise',
      detail: `Version: ${app.getVersion()}\n\nProfessional Accident Recreation Platform\n\n© 2024 AccuScene Enterprise. All rights reserved.`,
      buttons: ['OK'],
    });
  }

  private onWindowAllClosed(): void {
    if (process.platform !== 'darwin') {
      app.quit();
    }
  }

  private onActivate(): void {
    if (this.mainWindow === null) {
      this.createMainWindow();
    }
  }

  private onBeforeQuit(): void {
    if (this.apiServer) {
      this.apiServer.kill();
      this.apiServer = null;
    }
  }
}

new AccuSceneApplication();
