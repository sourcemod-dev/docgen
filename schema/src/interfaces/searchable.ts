import { Identifier } from './symbol';

export enum Part {
    Name,
    Parameter,
    Return,
}

export interface SearchResult {
    name: string;

    identifier: Identifier;

    part: Part;

    score: number;
}

export interface SearchOptions {
    weighted?: boolean;

    identifier?: Identifier;
}

export interface Searchable {
    search(needle: string, options?: SearchOptions): Promise<SearchResult[]>;
}
