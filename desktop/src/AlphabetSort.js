//the first sort, and the plainest fuji will have: the order javascript's own sort() gives names, capitals before lowercase and page10 before page9, with no locale and no opinion

export default function alphabetCompare(a, b) {//which of two entries comes first by name
	return a.name < b.name ? -1 : a.name > b.name ? 1 : 0//< compares text by its UTF-16 code units, which is the order a bare sort() gives
}
