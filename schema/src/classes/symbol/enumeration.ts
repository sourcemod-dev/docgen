import { Declaration } from './base';
import { IEnumeration, IEntry } from '../../interfaces';

export class Enumeration extends Declaration implements IEnumeration {
    /**
     * @brief Enum entries
     * @readonly
     */
     readonly entries: IEntry[];

    public constructor(enumeration: IEnumeration) {
        super(enumeration);

        this.entries = enumeration.entries;
    }
}
