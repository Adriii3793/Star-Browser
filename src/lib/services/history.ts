import {call} from './ipc';
import type { HistoryEntry } from '$lib/types';

export function recordVisit(url: string, title: string, query: string | null): Promise<void> {
    return call('record_visit',{url, title, query});
}

export function recentHistory(limit=20): Promise<HistoryEntry[]> {
    return call('recent_history', {limit});

}

export function searchHistory(term: string, limit = 20): Promise<HistoryEntry[]> {
    return call('search_history',{term, limit});
}

export function clearHistory(): Promise<void> {
    return call('clear_history');
}