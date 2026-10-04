interface Item {
    id: number;
    name: string;
}

const items: Item[] = [];
for (let i = 0; i < 1000; i++) {
    items.push({ id: i, name: "item" });
}
const s = JSON.stringify(items);
const back = JSON.parse(s) as Item[];
console.log(`count=${back.length} len=${JSON.stringify(back).length}`);
