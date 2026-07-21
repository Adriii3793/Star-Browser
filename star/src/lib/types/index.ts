export interface HistoryEntry {
    id: number;
    url: string;
    title: string;
    query: string | null;
    visitedAt: number;
    visitCount: number;
}