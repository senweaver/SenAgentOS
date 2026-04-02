import type { WsMessage } from '../types/api';
import { getToken } from './auth';
import { basePath } from './basePath';
import { generateUUID } from './uuid';

export type WsMessageHandler = (msg: WsMessage) => void;
export type WsOpenHandler = () => void;
export type WsCloseHandler = (ev: CloseEvent) => void;
export type WsErrorHandler = (ev: Event) => void;

export interface WebSocketClientOptions {
  /** Base URL override. Defaults to current host with ws(s) protocol. */
  baseUrl?: string;
  /** Delay in ms before attempting reconnect. Doubles on each failure up to maxReconnectDelay. */
  reconnectDelay?: number;
  /** Maximum reconnect delay in ms. */
  maxReconnectDelay?: number;
  /** Set to false to disable auto-reconnect. Default true. */
  autoReconnect?: boolean;
}

const DEFAULT_RECONNECT_DELAY = 1000;
const MAX_RECONNECT_DELAY = 30000;

export const SESSION_STORAGE_KEY = 'senagent_session_id';

/** Return a stable session ID, persisted in sessionStorage across reconnects. */
export function getOrCreateSessionId(): string {
  let id = sessionStorage.getItem(SESSION_STORAGE_KEY);
  if (!id) {
    id = generateUUID();
    sessionStorage.setItem(SESSION_STORAGE_KEY, id);
  }
  return id;
}

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private currentDelay: number;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private intentionallyClosed = false;

  public onMessage: WsMessageHandler | null = null;
  public onOpen: WsOpenHandler | null = null;
  public onClose: WsCloseHandler | null = null;
  public onError: WsErrorHandler | null = null;

  private readonly baseUrl: string;
  private readonly reconnectDelay: number;
  private readonly maxReconnectDelay: number;
  private readonly autoReconnect: boolean;

  constructor(options: WebSocketClientOptions = {}) {
    let defaultBase: string;
    const isDev = import.meta.env.DEV;

    if (isDev) {
      // In development, connect directly to the gateway port.
      // WebSocket is NOT subject to CORS — the browser allows cross-origin ws:// connections.
      // Only localhost→localhost, so no security concern.
      defaultBase = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//localhost:42617`;
    } else {
      // In production, use relative path — gateway serves everything
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      defaultBase = `${protocol}//${window.location.host}`;
    }

    this.baseUrl = options.baseUrl ?? defaultBase;
    this.reconnectDelay = options.reconnectDelay ?? DEFAULT_RECONNECT_DELAY;
    this.maxReconnectDelay = options.maxReconnectDelay ?? MAX_RECONNECT_DELAY;
    this.autoReconnect = options.autoReconnect ?? true;
    this.currentDelay = this.reconnectDelay;
  }

  /** Open the WebSocket connection. */
  connect(): void {
    // Prevent old socket's onclose from scheduling reconnect before we swap ws.
    this.intentionallyClosed = true;
    this.clearReconnectTimer();
    // Close any existing socket before reconnecting.
    // Prevents race between StrictMode double-mount: old socket's onerror/onclose
    // firing after the new socket has already opened (causes spurious error logs).
    if (this.ws) {
      const old = this.ws;
      this.ws = null;
      if (old.readyState !== WebSocket.CLOSED && old.readyState !== WebSocket.CLOSING) {
        old.close();
      }
    }
    this.intentionallyClosed = false;

    const token = getToken();
    const sessionId = getOrCreateSessionId();
    const params = new URLSearchParams();
    if (token) params.set('token', token);
    params.set('session_id', sessionId);
    const url = `${this.baseUrl}${basePath}/ws/chat?${params.toString()}`;

    const protocols: string[] = ['senagent.v1'];
    if (token) protocols.push(`bearer.${token}`);
    
    console.log('[WebSocket] Connecting to:', url);
    this.ws = new WebSocket(url, protocols);

    this.ws.onopen = () => {
      this.currentDelay = this.reconnectDelay;
      console.log('[WebSocket] Connected');
      this.onOpen?.();
    };

    this.ws.onmessage = (ev: MessageEvent) => {
      try {
        const msg = JSON.parse(ev.data) as WsMessage;
        this.onMessage?.(msg);
      } catch {
        // Ignore non-JSON frames
      }
    };

    this.ws.onclose = (ev: CloseEvent) => {
      console.log('[WebSocket] Closed:', ev.code, ev.reason);
      this.onClose?.(ev);
      this.scheduleReconnect();
    };

    this.ws.onerror = (ev: Event) => {
      console.error('[WebSocket] Error:', ev);
      this.onError?.(ev);
    };
  }

  /** Send a chat message to the agent. */
  sendMessage(content: string): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket is not connected');
    }
    this.ws.send(JSON.stringify({ type: 'message', content }));
  }

  /** Close the connection without auto-reconnecting. */
  disconnect(): void {
    this.intentionallyClosed = true;
    this.clearReconnectTimer();
    if (this.ws && this.ws.readyState !== WebSocket.CLOSED && this.ws.readyState !== WebSocket.CLOSING) {
      this.ws.close();
    }
    this.ws = null;
  }

  /** Returns true if the socket is open. */
  get connected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  // ---------------------------------------------------------------------------
  // Reconnection logic
  // ---------------------------------------------------------------------------

  private scheduleReconnect(): void {
    if (this.intentionallyClosed || !this.autoReconnect) return;

    this.reconnectTimer = setTimeout(() => {
      this.currentDelay = Math.min(this.currentDelay * 2, this.maxReconnectDelay);
      this.connect();
    }, this.currentDelay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }
}
