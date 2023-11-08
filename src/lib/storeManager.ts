// storeManager.ts
import { Store } from "tauri-plugin-store-api";

export class StoreManager {
  private store: Store;

  constructor(storeName: string) {
    this.store = new Store(storeName);
  }

  async get(key: string): Promise<any> {
    await this.store.load();
    return this.store.get(key);
  }

  async set(key: string, value: any): Promise<void> {
    await this.store.set(key, value);
    await this.store.save();
  }
}
