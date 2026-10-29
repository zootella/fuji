//the first sort, and the plainest fuji will have: the order javascript's own sort() gives names, capitals before lowercase and page10 before page9, with no locale and no opinion

export default function alphabetSort(entries) {//these entries in order by name, as a new array, whether a folder's files or the rows of a list; a sort returns the order rather than a comparator, so a shuffle can be one too, and it is stable, so entries it cannot tell apart keep the order they came in
	return [...entries].sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)//< compares text by its UTF-16 code units, which is the order a bare sort() gives
}
